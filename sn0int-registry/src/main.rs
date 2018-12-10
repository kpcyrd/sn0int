#![allow(proc_macro_derive_resolution_fallback)]
#![warn(unused_extern_crates)]
#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate maplit;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate failure;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;

use rocket_contrib::{Json, Value, Template};
use dotenv::dotenv;

use std::env;
use crate::errors::*;

pub mod assets;
pub mod auth;
pub mod auth2;
pub mod db;
pub mod errors;
pub mod github;
pub mod models;
pub mod routes;
#[allow(unused_imports)]
pub mod schema;


#[catch(400)]
fn bad_request() -> Json<Value> {
    Json(json!({
        "error": "Bad request"
    }))
}

#[catch(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "error": "Resource was not found"
    }))
}

#[catch(500)]
fn internal_error() -> Json<Value> {
    Json(json!({
        "error": "Internal server error"
    }))
}

fn run() -> Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL must be set")?;

    db::setup_db(&database_url, 60)
        .context("Failed to setup db")?;

    rocket::ignite()
        .manage(db::init(&database_url))
        .attach(Template::fairing())
        .mount("/api/v0", routes![
            routes::api::quickstart,
            routes::api::search,
            routes::api::info,
            routes::api::download,
            routes::api::publish,
            routes::api::whoami,
        ])
        .mount("/auth", routes![
            routes::auth::get,
            routes::auth::post,
            routes::auth::login,
        ])
        .mount("/", routes![
            routes::assets::index,
            routes::assets::favicon,
            routes::assets::style,
        ])
    .catch(catchers![
        bad_request,
        not_found,
        internal_error,
    ])
    .launch();

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        for cause in err.iter_chain().skip(1) {
            eprintln!("Because: {}", cause);
        }
        std::process::exit(1);
    }
}
