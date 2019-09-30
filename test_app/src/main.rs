extern crate steamworks;
extern crate hyper;
#[macro_use]
extern crate serde_derive;

use steamworks::user_stats::*;
use futures::executor::block_on;
use serde::Deserialize;
use futures_timer::Delay;

use hyper_tls::HttpsConnector;
use hyper::{Client, Request, Body, Response, StatusCode};
use futures_util::TryStreamExt;
use select::document::Document;
use select::predicate::{Class, Attr};
use tendril::SliceExt;
use hyper::client::HttpConnector;
use hyper::client::connect::dns::GaiResolver;
use std::time::{Instant, Duration};
use std::fs;

use std::error::Error;
use std::fmt;
use tendril::fmt::Slice;

use rusqlite::types::ToSql;
use rusqlite::{params, Connection};
use std::path::Path;

fn get_leaderboard_name_workshop(file_name_no_ext: &[u8], game_mode_id: u8, creator_id: &[u8]) -> Vec<u8> {
    return [file_name_no_ext, "_".as_bytes(), game_mode_id.to_string().as_bytes(), "_".as_bytes(), creator_id, "_stable".as_bytes()].concat();
}

fn get_leaderboard_name_official(file_name_no_ext: &[u8], game_mode_id: u8) -> Vec<u8> {
    return [file_name_no_ext, "_".as_bytes(), game_mode_id.to_string().as_bytes(), "_stable".as_bytes()].concat();
}

#[derive(Deserialize, Debug)]
struct SteamPublishedFileDetails {
    publishedfileid: String,
    title: Option<String>,
    filename: Option<String>,
    creator: Option<String>,
    result: u8,
}

#[derive(Deserialize, Debug)]
struct SteamPublishedFileDetailsList {
    result: usize,
    resultcount: usize,
    publishedfiledetails: Vec<SteamPublishedFileDetails>,
}

#[derive(Deserialize, Debug)]
struct SteamGetPublishedFileDetailsResponse {
    response: SteamPublishedFileDetailsList,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct StatusError {
    status_code: StatusCode,
}

impl fmt::Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A request failed: Status code {}", self.status_code.as_str()) // user-facing output
    }
}

impl fmt::Debug for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A request failed after timeout: Status code {:?}: {:?} {{ file: {}, line: {} }}", self.status_code.as_str(), self.status_code.canonical_reason(), file!(), line!())
    }
}

impl Error for StatusError {}

async fn get_level_details(levels: &Vec<String>, timeout_ms: u64) -> std::result::Result<Vec<SteamPublishedFileDetails>, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();

    let https = HttpsConnector::new().unwrap();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut req_body: String = format!("itemcount={}", levels.len().to_string());
    let mut index: usize = 0;
    for level_id in levels {
        req_body.push_str(&format!("&publishedfileids%5B{}%5D={}", index.to_string(), level_id));
        index += 1;
    }

    // this tells the compiler the correct error return type. compilation fails without it :shrug:
    // there is probably a better way to do this. if you see this, please message me how!
    if 6 < 5 {
        return Result::Err(Box::new(StatusError { status_code: StatusCode::from_u16(200).unwrap() }));
    }

    let response: Response<hyper::Body> = loop {

        let req = Request::builder()
            .method("POST")
            .uri("https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(req_body.clone()))
            .unwrap();

        let response = client.request(req).await;
        if response.is_err() {
            return std::result::Result::Err(Box::new(response.unwrap_err()));
        }
        let response = response.unwrap();

        if response.status().is_success() {
            break response;
        } else if start.elapsed() >= Duration::from_millis(timeout_ms) {
            return Result::Err(Box::new(StatusError { status_code: response.status() }));
        }

        Delay::new(Duration::from_secs(1)).await?
    };

    let body: hyper::Chunk = response.into_body().try_concat().await?;

    let json_res: SteamGetPublishedFileDetailsResponse = serde_json::from_slice(&body)?;

    Ok(json_res.response.publishedfiledetails)
}

fn get_level_filename_no_ext(filename: String) -> String {
    filename.trim_end_matches(".bytes").to_string()
}

async fn get_distance_workshop_page(page: u32) -> Result<Option<Vec<String>>> {
    let https = HttpsConnector::new().unwrap();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let res_fut = client.get(format!("https://steamcommunity.com/workshop/browse/?appid=233610&actualsort=mostrecent&browsesort=mostrecent&p={}", page).parse().unwrap());

    let response = res_fut.await?;
    let body: hyper::Chunk = response.into_body().try_concat().await?;
    //let body_bytes_u8 = body.into_bytes().to_vec();
    let body_str = String::from_utf8(body.into_bytes().to_vec()).unwrap();

    let document = Document::from(&body_str[..]);

    if document.find(Attr("id", "no_items")).next() != None {
        println!("no items!");
        return Ok(None);
    }

    Ok(Some(document.find(Class("ugc"))
        .map(|node| node.attr("data-publishedfileid"))
        .filter(Option::is_some)
        .map(|str_opt| String::from(str_opt.unwrap()))
        .collect::<Vec<String>>()))
}

struct DistanceWorkshopGetter {
    page_num: u32,
}

async fn next_getter_page(getter: &mut DistanceWorkshopGetter) -> Result<Option<Vec<String>>> {
    getter.page_num += 1;
    get_distance_workshop_page(getter.page_num).await
}

fn for_distance_workshop() -> DistanceWorkshopGetter {
    return DistanceWorkshopGetter{ page_num: 0};
}

struct DbPlayer {
    steam_id: String,
    score_sum: i32,
    score_count: i32,
}

struct DbLevel {
    level_id: String,
    score_sum: i32,
}

struct DbLeaderboardEntry {
    level_id: String,
    steam_id: String,
    time: i32,
    placement: i32,
    score: i32,
}

fn upsert_entry(connection: &Connection, level_id: String, steam_id: String, time: i32, name: String) -> Result<()> {
    connection.execute(
        "INSERT INTO players (steam_id, cached_display_name) VALUES (?1, ?2)
            ON CONFLICT(steam_id) DO NOTHING",
        params![steam_id, name],
    )?;
    connection.execute(
        "INSERT INTO levels (level_id) VALUES (?1)
	        ON CONFLICT(level_id) DO NOTHING",
        params![level_id],
    )?;
    connection.execute(
        "INSERT INTO steam_leaderboard (level_id, steam_id, time) VALUES (?1, ?2, ?3)
	        ON CONFLICT(level_id, steam_id) DO UPDATE SET time = excluded.time",
        params![level_id, steam_id, time],
    )?;
    Ok(())
}

async fn upsert_leaderboard(connection: &Connection, client: &steamworks::Client, entries: &Vec<LeaderboardEntry>, level_id_bytes: Vec<u8>) -> Result<()> {
    let level_id = String::from_utf8(level_id_bytes.clone())?;
    let mut counter = 0;
    for entry in entries {
        counter += 1;
        let name = entry.steam_id.persona_name(client).await;
        upsert_entry(connection, level_id.clone(), entry.steam_id.as_u64().to_string(), entry.score, name)?;
    }
    update_leaderboard_scores(connection, level_id);
    Ok(())
}

fn update_leaderboard_scores(connection: &Connection, level_id: String) -> Result<()> {
    connection.execute(
        "DROP TABLE IF EXISTS temp.row_nums;",
        params![],
    )?;
    connection.execute(
        "CREATE TEMPORARY TABLE row_nums AS
        SELECT
            steam_id
        FROM steam_leaderboard
        WHERE level_id = ?1
        ORDER BY time ASC;",
        params![level_id],
    )?;
    connection.execute(
        "UPDATE steam_leaderboard
        SET
            placement = (SELECT ROWID FROM temp.row_nums WHERE steam_id = steam_leaderboard.steam_id)
        WHERE
            level_id = ?1;",
        params![level_id],
    )?;
    connection.execute(
        "UPDATE steam_leaderboard
        SET
            score = ((SELECT score_count FROM levels WHERE level_id = steam_leaderboard.level_id) - placement + 1)
        WHERE
            level_id = ?1;",
        params![level_id],
    )?;
    Ok(())
}

fn update_level_score_sums(connection: &Connection) -> Result<()> {
    connection.execute(
    "UPDATE levels
    SET
        score_sum = (
            SELECT SUM(score_sum)
            FROM (SELECT (SELECT score_sum FROM players WHERE steam_id = steam_leaderboard.steam_id) AS score_sum FROM steam_leaderboard WHERE level_id = levels.level_id)
        )",
        params![],
    )?;
    Ok(())
}

fn update_file_id_name(connection: &Connection, level_id: String, file_id: String, name: String) -> Result<()> {
    connection.execute(
        "UPDATE levels
        SET
            file_id = ?2,
            cached_display_name = ?3
        WHERE level_id = ?1",
        params![level_id, file_id, name],
    )?;
    Ok(())
}

fn read_official_levels_list(file_location: String) -> Result<Vec<String>> {
    let file = fs::read_to_string(file_location)?;
    let json_res: std::result::Result<Vec<String>, serde_json::error::Error> = serde_json::from_str(&file);
    match json_res {
        Ok(levels) => Ok(levels),
        _ => Err(Box::new(json_res.unwrap_err())),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting");
    let mut connection = Connection::open(r#"D:\Development\Distance\ApiServer\test_app\test.db"#)?;
    /*
    let transaction = connection.transaction()?;
    upsert_leaderboard(&transaction, get_leaderboard_name_official("Cataclysm".as_bytes(), 1)).await?;
    println!("Updating level score_sums...");
    update_level_score_sums(&transaction)?;
    println!("Updated level score_sums.");
    transaction.commit();
    connection.close();
    println!("Done");
    */

    let client = steamworks::Client::init().unwrap();
    {
        let transaction = connection.transaction()?;
        let officials = read_official_levels_list(r#"D:\Development\Distance\ApiServer\test_app\official_levels.json"#.to_string())?;
        for filename in officials {
            let lb_name = get_leaderboard_name_official(filename.as_bytes(), 1);
            let leaderboard = client.find_leaderboard(lb_name.clone()).await;
            match leaderboard {
                Ok(leaderboard) => {
                    let entries: Vec<LeaderboardEntry> = leaderboard.download_global(1, 100000, 0).await;
                    println!("Updating leaderboard for {}", filename);
                    upsert_leaderboard(&transaction, &client, &entries, lb_name.clone()).await?;
                    update_file_id_name(&transaction, String::from_utf8(lb_name)?, "official".to_string(), filename);
                },
                NotFound => {
                    println!("No sprint leaderboard for {}", filename);
                },
                _ => {
                    leaderboard?;
                }
            }
        }
        transaction.commit();
    }

    let mut index: u32 = 0;
    let mut getter = for_distance_workshop();
    while let Some(page) = next_getter_page(&mut getter).await.unwrap() {
        println!("page {:?}",index + 1);
        let transaction = connection.transaction()?;
        let details = get_level_details(&page, 10000).await?;
        for detail in details {
            if detail.creator.is_some() && detail.filename.is_some() {
                let filename = get_level_filename_no_ext(detail.filename.unwrap());
                let creator = detail.creator.unwrap();
                let lb_name = get_leaderboard_name_workshop(filename.as_bytes(), 1, creator.as_bytes());
                let leaderboard = client.find_leaderboard(lb_name.clone()).await;
                match leaderboard {
                    Ok(leaderboard) => {
                        let entries: Vec<LeaderboardEntry> = leaderboard.download_global(1, 100000, 0).await;
                        println!("Updating leaderboard for {}", filename);
                        upsert_leaderboard(&transaction, &client, &entries, lb_name.clone()).await?;
                        update_file_id_name(&transaction, String::from_utf8(lb_name)?,detail.publishedfileid, detail.title.unwrap_or(filename));
                    },
                    NotFound => {
                        println!("No sprint leaderboard for {}", filename);
                    },
                    _ => {
                        leaderboard?;
                    }
                }
            }
        }
        transaction.commit();
        index += 1;
        if index > 1000 {
            break;
        }
    }
    println!("Updating level score_sums...");
    let transaction = connection.transaction()?;
    update_level_score_sums(&transaction)?;
    transaction.commit();
    println!("Updated level score_sums.");
    connection.close();
    println!("done");

    /*
    let mut index: u32 = 0;
    let mut getter = for_distance_workshop();
    while let Some(page) = next_getter_page(&mut getter).await.unwrap() {
        println!("page {:?}",page);
        let filenames = get_level_filenames(&page, 10).await;
        println!("{:?}", filenames);
        for id in page {
            //println!("{}",id);
        }
        index += 1;
        if index > 0 {
            break;
        }
    }
    println!("done");

    let client = steamworks::Client::init().unwrap();

    let lb_name = get_leaderboard_name_official("Cataclysm".as_bytes(), 1);

    let leaderboard: LeaderboardHandle = client.find_leaderboard(lb_name).await.unwrap();

    let entries = leaderboard.download_global(1, 100000, 0).await;

    let name = entries[0].steam_id.persona_name(&client).await;

    let mil = entries[0].score%1000;
    let sec = entries[0].score%(1000*60)/1000;
    let min = entries[0].score/(1000*60);

    println!("Rank: {} Score: {} ({}:{}.{}) Id: {}", entries[0].global_rank, entries[0].score, min,sec,mil, name);
    println!("Count: {}", entries.len());
    */
    Ok(())
}