#[macro_use]
extern crate log;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use std::borrow::Borrow;
use std::fs::File;
use std::process::Command;

use crate::database::{create_database, Video};
use r2d2_sqlite::SqliteConnectionManager;
use sha2::Digest;
use tokio::task::spawn_blocking;
use tower_http::services::ServeDir;

mod database;

type DbPool = r2d2::Pool<SqliteConnectionManager>;

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
        let hash = sha2::Sha256::digest(&t.as_bytes());
        let mut hash = format!("{:x}", hash);
        hash.truncate(10);
        hash
    }

    let scripts_dir = std::env::var("SCRIPTS_PATH").unwrap_or("./scripts/".to_string());

    let video_name = calculate_hash(video_url);
    let mut cmd = Command::new("./generic_vids.sh");
    cmd.args([video_url, output_path, &video_name]).current_dir(scripts_dir);
    info!("Video fetched: {:?}", cmd);
    cmd.output()?;

    let video_file = File::open(format!("{}/{}/{}.js", output_path, video_name, video_name))?;
    let video: Video = serde_json::from_reader(video_file)?;
    Ok(video)
}

async fn get_videos(State(db): State<DbPool>) -> Result<Json<Vec<Video>>, (StatusCode, String)> {
    database::list_videos(db.get().unwrap().borrow())
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn insert_video(
    State(db): State<DbPool>,
    Path(vid_url): Path<String>,
) -> Result<Json<Video>, (StatusCode, String)> {
    let video = spawn_blocking(move || {
        if vid_url.starts_with("https://coub.com") {
            let coub_name = vid_url.split('/').last().unwrap_or_default();
            fetch_coub(
                &coub_name,
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
    // Logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Database and connection pool setup
    let connspec = std::env::var("DATABASE_PATH").unwrap_or("db.sqlite".to_string());
    let manager = SqliteConnectionManager::file(connspec);
    let pool: DbPool = r2d2::Pool::new(manager).expect("Failed to create pool to sqlite database.");
    create_database(pool.get()?.borrow()).expect("Cannot create database schema");

    // API/Webservice setup
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let scripts_dir_path = std::env::var("SCRIPTS_PATH").unwrap_or("./scripts/".to_string());
    let videos_dir_path = std::env::var("VIDEOS_PATH").unwrap_or("./videos/".to_string());
    let webapp_dir_path = std::env::var("WEBAPP_PATH").unwrap_or("./dist/".to_string());
    info!("videos_dir_path: {}", videos_dir_path);
    info!("webapp_dir_path: {}", webapp_dir_path);
    info!("scripts_dir_path: {}", scripts_dir_path);

    //HttpServer::new(move || App::new()
    //    .wrap(middleware::Logger::default())
    //    .data(pool.clone())
    //    .data(web::JsonConfig::default().limit(4096))
    //    .service(get_videos)
    //    .service(add_video_tags)
    //    .service(insert_video)
    //    .service(fs::Files::new("/videos", &videos_dir_path).show_files_listing())
    //    .service(fs::Files::new("/", &webapp_dir_path))
    //)
    //    .bind(format!("[::]:{}", &port))?
    //    .run()
    //    .await

    // build our application with a route
    let app = Router::new()
        .route("/api/videos", get(get_videos))
        .route("/api/video/{name}", put(insert_video))
        .route("/api/video/{name}/tags", put(add_video_tags))
        .nest_service("/videos", ServeDir::new(videos_dir_path))
        .fallback_service(ServeDir::new(webapp_dir_path))
        .with_state(pool);

    // run it
    let listener = tokio::net::TcpListener::bind(format!("[::]:{}", port)).await?;

    info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
