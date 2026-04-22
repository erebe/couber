use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    pub name: String,
    pub tags: Vec<String>,
    pub url: String,
    pub original: String,
    pub thumbnail: String,
    pub creation_timestamp: u32,
}

pub async fn create_database(pool: &SqlitePool) -> sqlx::Result<()> {
    sqlx::query("CREATE TABLE IF NOT EXISTS videos (name TEXT PRIMARY KEY, url TEXT, tags JSON, original TEXT, thumbnail TEXT, creation_timestamp INTEGER)")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_videos(pool: &SqlitePool) -> sqlx::Result<Vec<Video>> {
    let rows =
        sqlx::query("SELECT name, url, tags, original, thumbnail, creation_timestamp FROM videos")
            .fetch_all(pool)
            .await?;

    let videos = rows
        .into_iter()
        .map(|row| {
            let tags_str: String = row.get("tags");
            Video {
                name: row.get("name"),
                url: row.get("url"),
                tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                original: row.get("original"),
                thumbnail: row.get("thumbnail"),
                creation_timestamp: row.get::<i64, _>("creation_timestamp") as u32,
            }
        })
        .collect();

    Ok(videos)
}

pub async fn insert_video(pool: &SqlitePool, video: &Video) -> sqlx::Result<()> {
    sqlx::query("INSERT OR REPLACE INTO videos (name, url, tags, original, thumbnail, creation_timestamp) VALUES (?,?,?,?,?,?)")
        .bind(&video.name)
        .bind(&video.url)
        .bind(serde_json::to_string(&video.tags.iter().collect::<HashSet<_>>()).unwrap_or_default())
        .bind(&video.original)
        .bind(&video.thumbnail)
        .bind(video.creation_timestamp as i64)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_tags(
    pool: &SqlitePool,
    video_name: &str,
    tags: &HashSet<String>,
) -> sqlx::Result<()> {
    sqlx::query("UPDATE videos SET tags = ? WHERE name = ?")
        .bind(serde_json::to_string(tags).unwrap_or_default())
        .bind(video_name)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_tag(pool: &SqlitePool, video_name: &str, tags: &Vec<String>) -> sqlx::Result<()> {
    sqlx::query(
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
    )
    .bind(video_name)
    .bind(serde_json::to_string(tags).unwrap_or_default())
    .bind(video_name)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::database::{add_tag, create_database, insert_video, list_videos, Video};
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;

    async fn setup() -> (SqlitePool, Video) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("Failed to create pool to sqlite database.");
        create_database(&pool)
            .await
            .expect("cannot create database schema");

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

    #[tokio::test]
    async fn test_create_database() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("Failed to create pool to sqlite database.");
        create_database(&pool)
            .await
            .expect("cannot create database schema");
    }

    #[tokio::test]
    async fn test_insert_videos() {
        let (pool, video) = setup().await;
        insert_video(&pool, &video)
            .await
            .expect("cannot insert video");
    }

    #[tokio::test]
    async fn test_list_videos() {
        let (pool, video) = setup().await;
        insert_video(&pool, &video)
            .await
            .expect("cannot insert video");

        assert_eq!(
            list_videos(&pool).await.expect("cannot list videos").len(),
            1
        );
    }

    #[tokio::test]
    async fn test_add_tags() {
        let (pool, video) = setup().await;
        insert_video(&pool, &video)
            .await
            .expect("cannot insert video");

        add_tag(
            &pool,
            video.name.as_str(),
            &vec![String::from("tags"), String::from("tags")],
        )
        .await
        .expect("cannot add tag");
        let vids = list_videos(&pool).await.expect("cannot list videos");
        assert_eq!(vids.len(), 1);
        assert_eq!(vids.first().unwrap().tags.len(), 1);
        assert_eq!(vids.first().unwrap().tags.first().unwrap(), "tags");
    }
}
