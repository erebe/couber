use std::borrow::Borrow;

use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{NO_PARAMS};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    pub name: String,
    pub tags: Vec<String>,
    pub url: String,
}


pub fn create_database(cnx: PooledConnection<SqliteConnectionManager>) -> rusqlite::Result<usize> {
    cnx.execute("CREATE TABLE IF NOT EXISTS videos (name TEXT PRIMARY KEY, url TEXT, tags JSON)", NO_PARAMS)
}

pub fn list_videos(cnx: &PooledConnection<SqliteConnectionManager>) -> rusqlite::Result<Vec<Video>> {
    let mut stmt = cnx.prepare("SELECT name, url, tags FROM videos")?;
    let videos = stmt.query_map(NO_PARAMS, |row| {
        Ok(Video {
            name: row.get(0)?,
            url: row.get(1)?,
            tags: serde_json::from_str((row.get::<usize, String>(2)?).borrow()).unwrap(),
        })
    })?
        .filter_map(|video| video.ok())
        .collect();

    Ok(videos)
}

pub fn insert_video(cnx: &PooledConnection<SqliteConnectionManager>, video: &Video) -> rusqlite::Result<()> {
    cnx.execute("INSERT OR REPLACE INTO videos (name, url, tags) VALUES (?,?,?)",
                &[&video.name, &video.url, &serde_json::to_string(&video.tags).unwrap()])?;
    Ok(())
}