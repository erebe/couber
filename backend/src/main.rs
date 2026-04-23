use axum::extract::{Form, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::Parser;
use eyre::WrapErr;
use maud::{html, Markup};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

use crate::database::create_database;
use crate::image_tagger::ImageTaggerService;
use crate::page_renderer::render_page;
use crate::video_fetcher::VideoFetcher;
use crate::video_store::VideoStore;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use tokio::task::spawn_blocking;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

mod database;
mod image_tagger;
mod page_renderer;
mod video_fetcher;
mod video_store;

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

    /// OpenRouter API key
    #[arg(long, env = "OPENROUTER_API_KEY")]
    openrouter_api_key: String,

    /// OpenRouter model to use for image tagging
    #[arg(long, env = "OPENROUTER_MODEL", default_value = "openrouter/auto")]
    openrouter_model: String,

    /// OpenRouter API URL
    #[arg(
        long,
        env = "OPENROUTER_API_URL",
        default_value = "https://openrouter.ai/api/v1/chat/completions"
    )]
    openrouter_api_url: String,
}

struct App {
    db: DbPool,
    videos_path: PathBuf,
    image_tagger: ImageTaggerService,
    video_fetcher: VideoFetcher,
    video_store: VideoStore,
}

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

async fn insert_video(State(app): State<Arc<App>>, Form(payload): Form<AddVideoForm>) -> Markup {
    let url = payload.url.trim().to_string();
    if url.is_empty() {
        return html! { span class="status-error" { "Please enter a URL." } };
    }

    let fetch_result = {
        let app = app.clone();
        spawn_blocking(move || {
            let (video, video_dir) = if url.starts_with("https://coub.com") {
                let coub_name = url.split('/').next_back().unwrap_or_default().to_string();
                app.video_fetcher.fetch_coub(&coub_name)?
            } else {
                app.video_fetcher.fetch_video(&url)?
            };
            app.video_store.add(&video.name, &video_dir)?;
            Ok::<_, eyre::Error>(video)
        })
        .await
    };

    let mut video = match fetch_result {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => return html! { span class="status-error" { "Error: " (e) } },
        Err(e) => return html! { span class="status-error" { "Error: " (e) } },
    };

    let thumbnail_path = app
        .videos_path
        .join(&video.name)
        .join(format!("{}.thumbnail.png", video.name));
    let tags = app.image_tagger.extract_image_tags(&thumbnail_path).await;
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
    let tags: HashSet<String> = payload
        .tags
        .split(',')
        .map(|t| urlencoding::encode(t.trim()).into_owned())
        .filter(|t| !t.is_empty())
        .collect();

    match database::set_tags(&app.db, &payload.name, &tags).await {
        Ok(_) => html! { span class="status-success" { "Tags saved!" } },
        Err(e) => html! { span class="status-error" { "Error: " (e) } },
    }
}

#[derive(Deserialize)]
struct DeleteVideoForm {
    name: String,
}

async fn delete_video(State(app): State<Arc<App>>, Form(payload): Form<DeleteVideoForm>) -> Markup {
    if let Err(e) = app.video_store.delete(&payload.name) {
        return html! { span class="status-error" { "Error: " (e) } };
    };

    match database::delete_video(&app.db, &payload.name).await {
        Ok(_) => html! { span class="status-success" { "Video deleted!" } },
        Err(e) => html! { span class="status-error" { "Error: " (e) } },
    }
}

#[derive(Deserialize)]
struct SuggestTagsQuery {
    name: String,
}

#[derive(Deserialize)]
struct NormalizeTagsRequest {
    tags: Vec<String>,
}

async fn normalize_tags(
    State(app): State<Arc<App>>,
    Json(payload): Json<NormalizeTagsRequest>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let normalized = app
        .image_tagger
        .normalize_tags(payload.tags)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(normalized))
}

async fn suggest_tags(
    State(app): State<Arc<App>>,
    axum::extract::Query(params): axum::extract::Query<SuggestTagsQuery>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let thumbnail_path = app
        .videos_path
        .join(&params.name)
        .join(format!("{}.thumbnail.png", params.name));
    let tags = app
        .image_tagger
        .extract_image_tags(&thumbnail_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let decoded: Vec<String> = tags.into_iter().map(|t| decode_tags(&t)).collect();
    Ok(Json(decoded))
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

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("INFO,tower_http=DEBUG")),
        )
        .with_ansi(true)
        .init();

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

    let video_store = VideoStore::new(videos_path.clone());
    let video_fetcher = VideoFetcher::new(scripts_path.clone());
    let image_tagger = ImageTaggerService::new(
        cli.openrouter_api_key,
        cli.openrouter_model,
        cli.openrouter_api_url,
    );
    let app = Arc::new(App {
        db: pool,
        videos_path,
        image_tagger,
        video_fetcher,
        video_store,
    });

    let router = Router::new()
        .route("/", get(index))
        .route("/add-video", post(insert_video))
        .route("/update-tags", post(update_tags_form))
        .route("/delete-video", post(delete_video))
        .route("/api/tags", get(list_tags))
        .route("/api/suggest-tags", get(suggest_tags))
        .route("/api/normalize-tags", post(normalize_tags))
        .nest_service("/videos", ServeDir::new(app.videos_path.clone()))
        .layer(TraceLayer::new_for_http())
        .with_state(app);

    let listener = tokio::net::TcpListener::bind(format!("[::]:{}", cli.port)).await?;
    info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}
