#[macro_use]
extern crate log;

use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use maud::{html, Markup};
use serde::Deserialize;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::fs::File;
use std::process::Command;

use crate::database::{create_database, Video};
use crate::page_renderer::render_page;
use r2d2_sqlite::SqliteConnectionManager;
use sha2::Digest;
use tokio::task::spawn_blocking;
use tower_http::services::ServeDir;

mod database;
mod page_renderer;

type DbPool = r2d2::Pool<SqliteConnectionManager>;

#[derive(serde::Deserialize)]
struct InsertVideoPayload {
    url: String,
}

fn fetch_coub(coub_name: &str, output_path: &str) -> eyre::Result<Video> {
    let scripts_dir = std::env::var("SCRIPTS_PATH").unwrap_or("./scripts/".to_string());
    let mut cmd = Command::new("./coub.sh");
    cmd.args([coub_name, output_path]).current_dir(scripts_dir);
    info!("Coub fetched: {:?}", cmd);
    cmd.output()?;
    let video_file = File::open(format!("{}/{}/{}.js", output_path, coub_name, coub_name))?;
    let video: Video = serde_json::from_reader(video_file)?;
    Ok(video)
}

fn fetch_video(video_url: &str, output_path: &str) -> eyre::Result<Video> {
    fn calculate_hash(t: &str) -> String {
        let hash = sha2::Sha256::digest(t.as_bytes());
        let mut hash = format!("{:x}", hash);
        hash.truncate(10);
        hash
    }
    let scripts_dir = std::env::var("SCRIPTS_PATH").unwrap_or("./scripts/".to_string());
    let video_name = calculate_hash(video_url);
    let mut cmd = Command::new("./generic_vids.sh");
    cmd.args([video_url, output_path, &video_name])
        .current_dir(scripts_dir);
    info!("Video fetched: {:?}", cmd);
    cmd.output()?;
    let video_file = File::open(format!("{}/{}/{}.js", output_path, video_name, video_name))?;
    let video: Video = serde_json::from_reader(video_file)?;
    Ok(video)
}

// --- Route handlers ---

async fn index(State(db): State<DbPool>) -> Result<Markup, (StatusCode, String)> {
    let mut videos = database::list_videos(db.get().unwrap().borrow())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    videos.sort_by_key(| b| std::cmp::Reverse(b.creation_timestamp));
    Ok(render_page(&videos))
}

#[derive(Deserialize)]
struct AddVideoForm {
    url: String,
}

async fn add_video_form(State(db): State<DbPool>, Form(payload): Form<AddVideoForm>) -> Markup {
    let url = payload.url.trim().to_string();
    if url.is_empty() {
        return html! { span class="status-error" { "Please enter a URL." } };
    }

    let result = spawn_blocking(move || {
        let videos_path = std::env::var("VIDEOS_PATH").unwrap_or("./videos/".to_string());
        let video = if url.starts_with("https://coub.com") {
            let coub_name = url.split('/').next_back().unwrap_or_default().to_string();
            fetch_coub(&coub_name, &videos_path)?
        } else {
            fetch_video(&url, &videos_path)?
        };
        database::insert_video(db.get().unwrap().borrow(), &video)
            .map_err(|e| eyre::eyre!(e.to_string()))?;
        Ok::<_, eyre::Error>(video)
    })
    .await;

    match result {
        Ok(Ok(_)) => html! { span class="status-success" { "Video added successfully!" } },
        Ok(Err(e)) => html! { span class="status-error" { "Error: " (e) } },
        Err(e) => html! { span class="status-error" { "Error: " (e) } },
    }
}

#[derive(Deserialize)]
struct UpdateTagsForm {
    name: String,
    tags: String,
}

async fn update_tags_form(State(db): State<DbPool>, Form(payload): Form<UpdateTagsForm>) -> Markup {
    let tags: Vec<String> = payload
        .tags
        .split(',')
        .map(|t| urlencoding::encode(t.trim()).into_owned())
        .filter(|t| !t.is_empty())
        .collect();

    let name = payload.name.clone();
    let result =
        spawn_blocking(move || database::set_tags(db.get().unwrap().borrow(), &name, &tags)).await;

    match result {
        Ok(Ok(_)) => html! { span class="status-success" { "Tags saved!" } },
        Ok(Err(e)) => html! { span class="status-error" { "Error: " (e) } },
        Err(e) => html! { span class="status-error" { "Error: " (e) } },
    }
}

fn decode_tags<T: AsRef<str>>(tags: T) -> String {
    urlencoding::decode(tags.as_ref())
        .unwrap_or_default()
        .into_owned()
}

async fn list_tags(
    State(db): State<DbPool>,
) -> Result<Json<HashSet<String>>, (StatusCode, String)> {
    let videos = database::list_videos(db.get().unwrap().borrow())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let tags: HashSet<String> = videos
        .into_iter()
        .flat_map(|v| v.tags)
        .map(decode_tags)
        .collect();
    Ok(Json(tags))
}

async fn insert_video(
    State(db): State<DbPool>,
    Json(payload): Json<InsertVideoPayload>,
) -> Result<Json<Video>, (StatusCode, String)> {
    let vid_url = payload.url;
    let video = spawn_blocking(move || {
        if vid_url.starts_with("https://coub.com") {
            let coub_name = vid_url.split('/').next_back().unwrap_or_default();
            fetch_coub(
                coub_name,
                &std::env::var("VIDEOS_PATH").unwrap_or("./videos/".to_string()),
            )
        } else {
            fetch_video(
                &vid_url,
                &std::env::var("VIDEOS_PATH").unwrap_or("./videos/".to_string()),
            )
        }
    })
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    database::insert_video(db.get().unwrap().borrow(), &video)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    Ok(Json(video))
}

async fn add_video_tags(
    State(db): State<DbPool>,
    Path(name): Path<String>,
    Json(tags): Json<Vec<String>>,
) -> Result<(), (StatusCode, String)> {
    spawn_blocking(move || database::add_tag(db.get().unwrap().borrow(), &name, &tags))
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let connspec = std::env::var("DATABASE_PATH").unwrap_or("db.sqlite".to_string());
    let manager = SqliteConnectionManager::file(connspec);
    let pool: DbPool = r2d2::Pool::new(manager).expect("Failed to create pool to sqlite database.");
    create_database(pool.get()?.borrow()).expect("Cannot create database schema");

    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let videos_dir_path = std::env::var("VIDEOS_PATH").unwrap_or("./videos/".to_string());
    let scripts_dir_path = std::env::var("SCRIPTS_PATH").unwrap_or("./scripts/".to_string());
    info!("videos_dir_path: {}", videos_dir_path);
    info!("scripts_dir_path: {}", scripts_dir_path);

    let app = Router::new()
        .route("/", get(index))
        .route("/add-video", post(add_video_form))
        .route("/update-tags", post(update_tags_form))
        .route("/api/tags", get(list_tags))
        .route("/api/video", put(insert_video))
        .route("/api/video/{name}/tags", put(add_video_tags))
        .nest_service("/videos", ServeDir::new(videos_dir_path))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(format!("[::]:{}", port)).await?;
    info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
