#[macro_use] extern crate log;

use std::borrow::Borrow;
use std::fs::File;
use std::process::Command;
use actix_files as fs;

use actix_web::{App, Error, get, HttpResponse, HttpServer, middleware, put, web};
use actix_web::rt::blocking::BlockingError;
use r2d2_sqlite::SqliteConnectionManager;

use crate::database::{create_database, Video};

mod database;

type DbPool = r2d2::Pool<SqliteConnectionManager>;


fn fetch_coub(coub_name: &str, output_path: &str) -> Result<Video, Error> {
    let scripts_dir = std::env::var("SCRIPTS_PATH").unwrap_or("./scripts/".to_string());

    let mut cmd = Command::new("./coub.sh");
    cmd.args(&[coub_name, output_path])
        .current_dir(scripts_dir);
    info!("Coub fetched: {:?}", cmd);
    cmd.output()?;

    let video_file = File::open(format!("{}/{}/{}.js", output_path, coub_name, coub_name))?;
    let video: Video = serde_json::from_reader(video_file)?;
    Ok(video)
}

#[get("/api/videos")]
async fn get_videos(db: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let res = web::block(move || {
        database::list_videos(db.get().unwrap().borrow())
    })
        .await
        .map(|video| HttpResponse::Ok().json(video))
        .map_err(|err| HttpResponse::InternalServerError().body(err.to_string()))?;
    Ok(res)
}

#[put("/api/video/{name}")]
async fn insert_video(path: web::Path<String>, db: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let res = web::block(move || {
        let video = fetch_coub(&path.0, std::env::var("VIDEOS_PATH").unwrap_or("./videos/".to_string()).as_str())
            .map_err(|err| BlockingError::Error(err.to_string()))?;
        database::insert_video(db.get().unwrap().borrow(), &video)
            .map_err(|err| BlockingError::Error(err.to_string()))
    })
        .await
        .map(|video| HttpResponse::Ok().json(video))
        .map_err(|err| {
            HttpResponse::InternalServerError().body(err.to_string())
        })?;
    Ok(res)
}

#[put("/api/video/tags/{name}")]
async fn add_video_tags(path: web::Path<String>, tags: web::Json<Vec<String>>, db: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let res = web::block(move || {
        database::add_tag(db.get().unwrap().borrow(), &path.0, &tags.0)
            .map_err(|err| BlockingError::Error(err.to_string()))
    })
        .await
        .map(|_| HttpResponse::Ok().body(""))
        .map_err(|err| {
            HttpResponse::InternalServerError().body(err.to_string())
        })?;
    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Database and connection pool setup
    let connspec = std::env::var("DATABASE_PATH").unwrap_or("db.sqlite".to_string());
    let manager = SqliteConnectionManager::file(connspec);
    let pool: DbPool = r2d2::Pool::new(manager)
        .expect("Failed to create pool to sqlite database.");
    create_database(pool.get().unwrap().borrow()).expect("Cannot create database schema");

    // API/Webservice setup
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let scripts_dir_path = std::env::var("SCRIPTS_PATH").unwrap_or("./scripts/".to_string());
    let videos_dir_path = std::env::var("VIDEOS_PATH").unwrap_or("./videos/".to_string());
    let webapp_dir_path = std::env::var("WEBAPP_PATH").unwrap_or("./dist/".to_string());
    info!("videos_dir_path: {}", videos_dir_path);
    info!("webapp_dir_path: {}", webapp_dir_path);
    info!("scripts_dir_path: {}", scripts_dir_path);

    HttpServer::new(move || App::new()
        .wrap(middleware::Logger::default())
        .data(pool.clone())
        .data(web::JsonConfig::default().limit(4096))
        .service(get_videos)
        .service(add_video_tags)
        .service(insert_video)
        .service(fs::Files::new("/videos", &videos_dir_path).show_files_listing())
        .service(fs::Files::new("/", &webapp_dir_path))
    )
        .bind(format!("[::]:{}", &port))?
        .run()
        .await
}
