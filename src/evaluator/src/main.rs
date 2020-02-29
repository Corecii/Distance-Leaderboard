extern crate steamworks;
#[macro_use]
extern crate serde_derive;

use steamworks::user_stats::*;
use steamworks::Client;
use rusqlite::{params, Connection};
use std::fs;

mod workshop;
mod db_create;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct App {
    steamworks: Client,
    database: Connection,
}

impl App {
    async fn new(db_file: &str) -> Result<App> {
        let connection = Connection::open(db_file)?;
        let client = steamworks::Client::init()?;
        Ok(App {
            steamworks: client,
            database: connection,
        })
    }

    async fn upsert_entry(connection: &Connection, level_id: String, steam_id: String, score: i32, name: String) -> Result<()> {
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
            "INSERT INTO steam_leaderboard (level_id, steam_id, score) VALUES (?1, ?2, ?3)
                ON CONFLICT(level_id, steam_id) DO UPDATE SET score = excluded.score",
            params![level_id, steam_id, score],
        )?;
        Ok(())
    }

    async fn update_leaderboard_evaluations(connection: &Connection, level_id: String) -> Result<()> {
        connection.execute(
            "DROP TABLE IF EXISTS temp.row_nums;",
            params![],
        )?;
        connection.execute(
            "CREATE TEMPORARY TABLE row_nums AS
            SELECT steam_id
            FROM steam_leaderboard
            WHERE level_id = ?1
            ORDER BY score ASC;",
            params![level_id],
        )?;
        connection.execute(
            "UPDATE steam_leaderboard
            SET placement = (SELECT ROWID FROM temp.row_nums WHERE steam_id = steam_leaderboard.steam_id)
            WHERE level_id = ?1;",
            params![level_id],
        )?;
        connection.execute(
            "UPDATE steam_leaderboard
            SET evaluation = ((SELECT score_count FROM levels WHERE level_id = steam_leaderboard.level_id) - placement*2)
            WHERE level_id = ?1;",
            params![level_id],
        )?;

        Ok(())
    }
    
    async fn upsert_leaderboard(connection: &Connection, steamworks: &steamworks::Client, entries: &Vec<LeaderboardEntry>, level_id_bytes: Vec<u8>) -> Result<()> {
        let level_id = String::from_utf8(level_id_bytes.clone())?;
        for entry in entries {
            let name = entry.steam_id.persona_name(steamworks).await;
            App::upsert_entry(&connection, level_id.clone(), entry.steam_id.as_u64().to_string(), entry.score, name).await?;
        }
        App::update_leaderboard_evaluations(&connection, level_id).await?;
        Ok(())
    }

    async fn update_level_score_sums(connection: &Connection) -> Result<()> {
        connection.execute(
        "UPDATE levels
        SET
            evaluation_sum = (
                SELECT SUM(evaluation_sum)
                FROM (SELECT (SELECT evaluation_sum FROM players WHERE steam_id = steam_leaderboard.steam_id) AS evaluation_sum FROM steam_leaderboard WHERE level_id = levels.level_id)
            )",
            params![],
        )?;
        Ok(())
    }

    async fn update_file_id_name(connection: &Connection, level_id: String, file_id: String, name: String) -> Result<()> {
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

    async fn upsert_leaderboard_player_to_player_entries(connection: &Connection, level_id: String) -> Result<()> {
        connection.execute(
            "DELETE FROM player_vs_player
            WHERE level_id = ?1;",
            params![level_id],
        )?;

        let mut statement = connection.prepare(
            "SELECT steam_id
            FROM steam_leaderboard
            WHERE level_id = ?1
            ORDER BY placement ASC"
        )?;
        let entries_iter = statement.query_map(params![level_id], |row| {
            Ok(row.get(0)?)
        })?;
        let entries: Vec<String> = entries_iter.map(|entry| entry.unwrap()).collect();
        for winner_index in 0..entries.len() {
            let winner_id = &entries[winner_index];
            for loser_index in (winner_index + 1)..entries.len() {
                let loser_id = &entries[loser_index];
                connection.execute(
                    "INSERT INTO player_vs_player (level_id, winner_id, loser_id) VALUES (?1, ?2, ?3)",
                    params![level_id, winner_id, loser_id],
                )?;
            }
        }

        Ok(())
    }
}

fn get_level_filename_no_ext(filename: String) -> String {
    filename.trim_end_matches(".bytes").to_string()
}

fn read_official_levels_list(file_location: &str) -> Result<Vec<String>> {
    let file = fs::read_to_string(file_location)?;
    let json_res: std::result::Result<Vec<String>, serde_json::error::Error> = serde_json::from_str(&file);
    match json_res {
        Ok(levels) => Ok(levels),
        _ => Err(Box::new(json_res.unwrap_err())),
    }
}

fn get_leaderboard_name_workshop(file_name_no_ext: &[u8], game_mode_id: u8, creator_id: &[u8]) -> Vec<u8> {
    return [file_name_no_ext, "_".as_bytes(), game_mode_id.to_string().as_bytes(), "_".as_bytes(), creator_id, "_stable".as_bytes()].concat();
}

fn get_leaderboard_name_official(file_name_no_ext: &[u8], game_mode_id: u8) -> Vec<u8> {
    return [file_name_no_ext, "_".as_bytes(), game_mode_id.to_string().as_bytes(), "_stable".as_bytes()].concat();
}

async fn update_certain_leaderboard(
    connection: &Connection,
    steamworks: &steamworks::Client,
    lb_name: Vec<u8>,
    file_id: String,
    name: String
) -> Result<()> {
    let leaderboard = steamworks.find_leaderboard(lb_name.clone()).await;
    match leaderboard {
        Ok(leaderboard) => {
            let entries: Vec<LeaderboardEntry> = leaderboard.download_global(1, 1000000, 0).await;
            println!("Updating leaderboard for {}", name);
            App::upsert_leaderboard(connection, steamworks, &entries, lb_name.clone()).await?;
            App::update_file_id_name(connection, String::from_utf8(lb_name)?, file_id, name).await?;
        },
        Err(e) => {
            println!("No sprint leaderboard for {} ({:?})", name, e);
        }
    }
    Ok(())
}

async fn run_app(app: &mut App, file_officials: Option<&str>) -> Result<()> {
    if file_officials.is_some() {
        let officials = read_official_levels_list(file_officials.unwrap());
        if officials.is_ok() {
            let officials = officials.unwrap();
            let transaction = app.database.transaction()?;
            for filename in officials {
                let lb_name = get_leaderboard_name_official(filename.as_bytes(), 1);
                update_certain_leaderboard(
                    &transaction,
                    &app.steamworks,
                    lb_name,
                    "official".to_string(),
                    filename
                ).await?;
            }
            transaction.commit()?;
        }
    }

    let mut index: u32 = 0;
    let mut getter = workshop::for_distance_workshop();
    while let Some(page) = getter.next_getter_page().await.unwrap() {
        println!("page {:?}",index + 1);
        let transaction = app.database.transaction()?;
        let details = workshop::get_level_details(&page, 10000).await?;
        for detail in details {
            if detail.creator.is_some() && detail.filename.is_some() {
                let filename = get_level_filename_no_ext(detail.filename.unwrap());
                let creator = detail.creator.unwrap();
                let lb_name = get_leaderboard_name_workshop(filename.as_bytes(), 1, creator.as_bytes());
                update_certain_leaderboard(
                    &transaction,
                    &app.steamworks,
                    lb_name,
                    detail.publishedfileid,
                    detail.title.unwrap_or(filename)
                ).await?;
            }
        }
        transaction.commit()?;
        index += 1;
        if index > 1000 {
            break;
        }
    }

    println!("Updating level score_sums...");
    let transaction = app.database.transaction()?;
    App::update_level_score_sums(&transaction).await?;
    transaction.commit()?;
    println!("Updated level score_sums.");

    Ok(())
}

#[tokio::main]
async fn main() {
    println!("Starting");
    let matches = clap::App::new("Distance Leaderboard Evaluator")
        .version("0.2.0")
        .author("Corecii <corecii@corecii.com>")
        .about("Downloads Distance leaderboard from Steam and evaluates it")
        .arg(clap::Arg::with_name("file-db")
            .long("file-db")
            .value_name("FILE")
            .help("Sets the sqlite db file to write the database to")
            .takes_value(true))
        .arg(clap::Arg::with_name("file-officials")
            .long("file-officials")
            .value_name("FILE")
            .help("Sets the json file to read official levels from")
            .takes_value(true))
        .get_matches();
    
    let file_db = matches.value_of("file-db").unwrap_or("distance_leaderboard.db");
    let file_officials = matches.value_of("file-officials");
    println!("Writing database to {}", file_db);

    let app = App::new(file_db).await;
    if app.is_err() {
        println!("Failed to start because {:?}", app.unwrap_err());
        std::process::exit(1);
    }

    let mut app = app.unwrap();

    let statements = db_create::get_statements();

    for statement in statements {
        let result = app.database.execute_batch(statement);
        if result.is_err() {
            println!("Failed because {:?}", result.unwrap_err());
            std::process::exit(1);
        }
    }

    let result = run_app(&mut app, file_officials).await;
    if result.is_err() {
        println!("Failed because {:?}", result.unwrap_err());
        std::process::exit(1);
    }
    
    let result = app.database.close();
    if result.is_err() {
        println!("Failed because {:?}", result.unwrap_err());
        std::process::exit(1);
    }

    println!("Done");
}