extern crate steamworks;
#[macro_use]
extern crate serde_derive;

use tokio::prelude::*;
use steamworks::user_stats::*;
use steamworks::Client;
use rusqlite::{params, Connection};
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct DbLeaderboardEntry {
    level_id: String,
    steam_id: String,
    time: i32,
    placement: i32,
    score: i32,
}

struct DbLeaderboardEntryLite {
    steam_id: String,
    placement: i32,
}

struct DbPlayerToPlayerEntry {
    level_id: String,
    winner_id: String,
    loser_id: String,
}

struct App {
    steamworks: Client,
    database: Connection,
}

impl App {
    async fn init<T>(&mut self, db_file: &str) -> Result<()> {
        let connection = Connection::open(db_file)?;
        self.database = connection;
        let client = steamworks::Client::init()?;
        self.steamworks = client;
        Ok(())
    }
    async fn upsert_entry(&self, level_id: String, steam_id: String, score: i32, name: String) -> Result<()> {
        self.database.execute(
            "INSERT INTO players (steam_id, cached_display_name) VALUES (?1, ?2)
                ON CONFLICT(steam_id) DO NOTHING",
            params![steam_id, name],
        )?;
        self.database.execute(
            "INSERT INTO levels (level_id) VALUES (?1)
                ON CONFLICT(level_id) DO NOTHING",
            params![level_id],
        )?;
        self.database.execute(
            "INSERT INTO steam_leaderboard (level_id, steam_id, score) VALUES (?1, ?2, ?3)
                ON CONFLICT(level_id, steam_id) DO UPDATE SET score = excluded.time",
            params![level_id, steam_id, score],
        )?;
        Ok(())
    }
    
    async fn upsert_leaderboard(&self, entries: &Vec<LeaderboardEntry>, level_id_bytes: Vec<u8>) -> Result<()> {
        let level_id = String::from_utf8(level_id_bytes.clone())?;
        for entry in entries {
            let name = entry.steam_id.persona_name(&self.steamworks).await;
            self.upsert_entry(level_id.clone(), entry.steam_id.as_u64().to_string(), entry.score, name).await?;
        }
        self.update_leaderboard_scores(level_id).await?;
        Ok(())
    }

    async fn update_leaderboard_scores(&self, level_id: String) -> Result<()> {
        self.database.execute(
            "DROP TABLE IF EXISTS temp.row_nums;",
            params![],
        )?;
        self.database.execute(
            "CREATE TEMPORARY TABLE row_nums AS
            SELECT steam_id
            FROM steam_leaderboard
            WHERE level_id = ?1
            ORDER BY score ASC;",
            params![level_id],
        )?;
        self.database.execute(
            "UPDATE steam_leaderboard
            SET placement = (SELECT ROWID FROM temp.row_nums WHERE steam_id = steam_leaderboard.steam_id)
            WHERE level_id = ?1;",
            params![level_id],
        )?;

        self.database.execute(
            "DELETE FROM player_vs_player
            WHERE level_id = ?1;",
            params![level_id],
        )?;
        let mut statement = self.database.prepare(
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
                self.database.execute(
                    "INSERT INTO player_vs_player (level_id, winner_id, loser_id) VALUES (?1, ?2, ?3)",
                    params![level_id, winner_id, loser_id],
                )?;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    println!("Starting");

    println!("Done");
}