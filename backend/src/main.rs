#[macro_use]
extern crate log;

use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use clap::Parser;
use eyre::WrapErr;
use maud::{html, Markup};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use crate::database::{create_database, Video};
use crate::image_tagger::extract_image_tags;
use crate::page_renderer::render_page;
use sha2::Digest;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use tokio::task::spawn_blocking;
use tower_http::services::ServeDir;

mod database;
mod image_tagger;
mod page_renderer;

type DbPool = SqlitePool;

#[derive(Parser)]
#[command(about = "Coub video archive server")]
struct Cli {
    /// Path to the SQLite database file
    #[arg(long, env = "DATABASE_PATH", default_value = "db.sqlite")]
    database_path: PathBuf,

    /// Directory where videos are stored
    #[arg(long, env = "VIDEOS_PATH", default_value = "./videos/")]
    videos_path: PathBuf,

    /// Directory where scripts are located
    #[arg(long, env = "SCRIPTS_PATH", default_value = "./scripts/")]
    scripts_path: PathBuf,

    /// Port to listen on
    #[arg(long, env = "PORT", default_value = "8080")]
    port: u16,
}

struct App {
    db: DbPool,
    videos_path: PathBuf,
    scripts_path: PathBuf,
}

#[derive(serde::Deserialize)]
struct InsertVideoPayload {
    url: String,
}

fn fetch_coub(
    coub_name: &str,
    output_path: &std::path::Path,
    scripts_path: &std::path::Path,
) -> eyre::Result<Video> {
    let mut cmd = Command::new("./coub.sh");
    cmd.args([coub_name])
        .arg(output_path)
        .current_dir(scripts_path);
    info!("Coub fetched: {:?}", cmd);
    cmd.output()?;
    let video_file = File::open(
        output_path
            .join(coub_name)
            .join(format!("{}.js", coub_name)),
    )?;
    let video: Video = serde_json::from_reader(video_file)?;
    Ok(video)
}

fn fetch_video(
    video_url: &str,
    output_path: &std::path::Path,
    scripts_path: &std::path::Path,
) -> eyre::Result<Video> {
    fn calculate_hash(t: &str) -> String {
        let hash = sha2::Sha256::digest(t.as_bytes());
        let mut hash = hex::encode(hash);
        hash.truncate(10);
        hash
    }
    let video_name = calculate_hash(video_url);
    let mut cmd = Command::new("./generic_vids.sh");
    cmd.arg(video_url)
        .arg(output_path)
        .arg(&video_name)
        .current_dir(scripts_path);
    info!("Video fetched: {:?}", cmd);
    cmd.output()?;
    let video_file = File::open(
        output_path
            .join(&video_name)
            .join(format!("{}.js", video_name)),
    )?;
    let video: Video = serde_json::from_reader(video_file)?;
    Ok(video)
}

// --- Route handlers ---

async fn index(State(app): State<Arc<App>>) -> Result<Markup, (StatusCode, String)> {
    let mut videos = database::list_videos(&app.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    videos.sort_by_key(|b| std::cmp::Reverse(b.creation_timestamp));
    Ok(render_page(&videos))
}

#[derive(Deserialize)]
struct AddVideoForm {
    url: String,
}

async fn add_video_form(State(app): State<Arc<App>>, Form(payload): Form<AddVideoForm>) -> Markup {
    let url = payload.url.trim().to_string();
    if url.is_empty() {
        return html! { span class="status-error" { "Please enter a URL." } };
    }

    let videos_path = app.videos_path.clone();
    let scripts_path = app.scripts_path.clone();
    let fetch_result = spawn_blocking(move || {
        let video = if url.starts_with("https://coub.com") {
            let coub_name = url.split('/').next_back().unwrap_or_default().to_string();
            fetch_coub(&coub_name, &videos_path, &scripts_path)?
        } else {
            fetch_video(&url, &videos_path, &scripts_path)?
        };
        Ok::<_, eyre::Error>(video)
    })
    .await;

    let mut video = match fetch_result {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => return html! { span class="status-error" { "Error: " (e) } },
        Err(e) => return html! { span class="status-error" { "Error: " (e) } },
    };

    let thumbnail_path = app
        .videos_path
        .join(&video.name)
        .join(format!("{}.thumbnail.png", video.name));
    let tags = extract_image_tags(&thumbnail_path).await;
    info!("Tags fetched for {}: {:?}", video.name, tags);
    video.tags.extend(tags.unwrap_or_default().into_iter());
    match database::insert_video(&app.db, &video).await {
        Ok(_) => html! { span class="status-success" { "Video added successfully!" } },
        Err(e) => html! { span class="status-error" { "Error: " (e) } },
    }
}

#[derive(Deserialize)]
struct UpdateTagsForm {
    name: String,
    tags: String,
}

async fn update_tags_form(
    State(app): State<Arc<App>>,
    Form(payload): Form<UpdateTagsForm>,
) -> Markup {
    let tags: Vec<String> = payload
        .tags
        .split(',')
        .map(|t| urlencoding::encode(t.trim()).into_owned())
        .filter(|t| !t.is_empty())
        .collect();

    let thumbnail_path = app
        .videos_path
        .join(&payload.name)
        .join(format!("{}.thumbnail.png", payload.name));
    let ntags = extract_image_tags(&thumbnail_path).await;
    info!("Tags fetched for {}: {:?}", payload.name, ntags);
    let tags = tags
        .into_iter()
        .chain(ntags.unwrap_or_default())
        .collect::<HashSet<String>>();

    match database::set_tags(&app.db, &payload.name, &tags).await {
        Ok(_) => html! { span class="status-success" { "Tags saved!" } },
        Err(e) => html! { span class="status-error" { "Error: " (e) } },
    }
}

fn decode_tags<T: AsRef<str>>(tags: T) -> String {
    urlencoding::decode(tags.as_ref())
        .unwrap_or_default()
        .into_owned()
}

async fn list_tags(
    State(app): State<Arc<App>>,
) -> Result<Json<HashSet<String>>, (StatusCode, String)> {
    let videos = database::list_videos(&app.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let tags: HashSet<String> = videos
        .into_iter()
        .flat_map(|v| v.tags)
        .map(decode_tags)
        .collect();
    Ok(Json(tags))
}

async fn insert_video(
    State(app): State<Arc<App>>,
    Json(payload): Json<InsertVideoPayload>,
) -> Result<Json<Video>, (StatusCode, String)> {
    let vid_url = payload.url;
    let videos_path = app.videos_path.clone();
    let scripts_path = app.scripts_path.clone();
    let video = spawn_blocking(move || {
        if vid_url.starts_with("https://coub.com") {
            let coub_name = vid_url.split('/').next_back().unwrap_or_default();
            fetch_coub(coub_name, &videos_path, &scripts_path)
        } else {
            fetch_video(&vid_url, &videos_path, &scripts_path)
        }
    })
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    database::insert_video(&app.db, &video)
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    Ok(Json(video))
}

async fn add_video_tags(
    State(app): State<Arc<App>>,
    Path(name): Path<String>,
    Json(tags): Json<Vec<String>>,
) -> Result<(), (StatusCode, String)> {
    database::add_tag(&app.db, &name, &tags)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let cli = Cli::parse();

    let options = SqliteConnectOptions::new()
        .filename(&cli.database_path)
        .create_if_missing(true);
    let pool: DbPool = SqlitePool::connect_with(options)
        .await
        .expect("Failed to create pool to sqlite database.");
    create_database(&pool)
        .await
        .expect("Cannot create database schema");

    let videos_path =
        std::fs::canonicalize(&cli.videos_path).wrap_err("Cannot canonicalize videos-path")?;
    let scripts_path =
        std::fs::canonicalize(&cli.scripts_path).wrap_err("Cannot canonicalize scripts-path")?;
    info!("videos_path: {}", videos_path.display());
    info!("scripts_path: {}", scripts_path.display());

    let app = Arc::new(App {
        db: pool,
        videos_path,
        scripts_path,
    });

    let router = Router::new()
        .route("/", get(index))
        .route("/add-video", post(add_video_form))
        .route("/update-tags", post(update_tags_form))
        .route("/api/tags", get(list_tags))
        .route("/api/video", put(insert_video))
        .route("/api/video/{name}/tags", put(add_video_tags))
        .nest_service("/videos", ServeDir::new(app.videos_path.clone()))
        .with_state(app);

    let listener = tokio::net::TcpListener::bind(format!("[::]:{}", cli.port)).await?;
    info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}
