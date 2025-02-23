use std::borrow::Borrow;

use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::NO_PARAMS;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    pub name: String,
    pub tags: Vec<String>,
    pub url: String,
    pub original: String,
    pub thumbnail: String,
    pub creation_timestamp: u32,
}

pub fn create_database(cnx: &PooledConnection<SqliteConnectionManager>) -> rusqlite::Result<usize> {
    cnx.execute("CREATE TABLE IF NOT EXISTS videos (name TEXT PRIMARY KEY, url TEXT, tags JSON, original TEXT, thumbnail TEXT, creation_timestamp INTEGER)", NO_PARAMS)
}

pub fn list_videos(
    cnx: &PooledConnection<SqliteConnectionManager>,
) -> rusqlite::Result<Vec<Video>> {
    let mut stmt =
        cnx.prepare("SELECT name, url, tags, original, thumbnail, creation_timestamp FROM videos")?;
    let videos = stmt
        .query_map(NO_PARAMS, |row| {
            Ok(Video {
                name: row.get(0)?,
                url: row.get(1)?,
                tags: serde_json::from_str((row.get::<usize, String>(2)?).borrow()).unwrap(),
                original: row.get(3)?,
                thumbnail: row.get(4)?,
                creation_timestamp: row.get(5)?,
            })
        })?
        .filter_map(|video| video.ok())
        .collect();

    Ok(videos)
}

pub fn insert_video(
    cnx: &PooledConnection<SqliteConnectionManager>,
    video: &Video,
) -> rusqlite::Result<()> {
    cnx.execute("INSERT OR REPLACE INTO videos (name, url, tags, original, thumbnail, creation_timestamp) VALUES (?,?,?,?,?,?)",
                &[&video.name, &video.url, &serde_json::to_string(&video.tags).unwrap(), &video.original, &video.thumbnail, video.creation_timestamp.to_string().as_str()])?;
    Ok(())
}

pub fn add_tag(
    cnx: &PooledConnection<SqliteConnectionManager>,
    video_name: &str,
    tags: &Vec<String>,
) -> rusqlite::Result<()> {
    //tags.iter().try_for_each(|tag| {
    cnx.execute(
        r#"
        UPDATE videos
        SET tags = (
            SELECT JSON_GROUP_ARRAY(DISTINCT value)
            FROM (
                SELECT value
                FROM json_each(videos.tags)
                where name = ?
                UNION
                SELECT value
                FROM json_each(?)
            )
        ) where name = ?"#,
        &[
            video_name,
            &serde_json::to_string(tags).unwrap_or_default(),
            video_name,
        ],
    )?;
    Ok(())
    //})
}

#[cfg(test)]
mod tests {
    use crate::database::{add_tag, create_database, insert_video, list_videos, Video};
    use crate::DbPool;
    use r2d2_sqlite::SqliteConnectionManager;
    use std::borrow::Borrow;

    fn setup() -> (DbPool, Video) {
        let manager = SqliteConnectionManager::memory();
        let pool: DbPool =
            r2d2::Pool::new(manager).expect("Failed to create pool to sqlite database.");
        create_database(pool.get().unwrap().borrow()).expect("cannot create database schema");

        (
            pool,
            Video {
                thumbnail: String::from("thumbnail.png"),
                original: String::from("ori.mp4"),
                tags: vec![],
                creation_timestamp: 12345,
                name: String::from("toto"),
                url: String::from("video.mp4"),
            },
        )
    }

    #[test]
    fn test_create_database() {
        let manager = SqliteConnectionManager::memory();
        let pool: DbPool =
            r2d2::Pool::new(manager).expect("Failed to create pool to sqlite database.");
        create_database(pool.get().unwrap().borrow()).expect("cannot create database schema");
    }

    #[test]
    fn test_insert_videos() {
        let (pool, video) = setup();
        insert_video(pool.get().unwrap().borrow(), &video).expect("cannot insert video");
    }

    #[test]
    fn test_list_videos() {
        let (pool, video) = setup();
        insert_video(pool.get().unwrap().borrow(), &video).expect("cannot insert video");

        assert_eq!(
            list_videos(pool.get().unwrap().borrow())
                .expect("cannot list videos")
                .len(),
            1
        );
    }

    #[test]
    fn test_add_tags() {
        let (pool, video) = setup();
        insert_video(pool.get().unwrap().borrow(), &video).expect("cannot insert video");

        add_tag(
            pool.get().unwrap().borrow(),
            video.name.as_str(),
            vec![String::from("tags"), String::from("tags")].borrow(),
        )
        .expect("cannot add tag");
        let vids = list_videos(pool.get().unwrap().borrow()).expect("cannot list videos");
        assert_eq!(vids.len(), 1);
        assert_eq!(vids.first().unwrap().tags.len(), 1);
        assert_eq!(vids.first().unwrap().tags.first().unwrap(), "tags");
    }
}
