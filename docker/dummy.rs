#[macro_use] extern crate log;

use std::borrow::Borrow;
use std::fs::File;
use std::process::Command;
use actix_files as fs;

use actix_web::{App, Error, get, HttpResponse, HttpServer, middleware, put, web};
use actix_web::rt::blocking::BlockingError;
use r2d2_sqlite::SqliteConnectionManager;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
 Ok(())
}
