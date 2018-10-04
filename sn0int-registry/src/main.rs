#![allow(proc_macro_derive_resolution_fallback)]
#![warn(unused_extern_crates)]
#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate sn0int_common;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate dotenv;
// extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate failure;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
extern crate diesel_full_text_search;
extern crate oauth2;
extern crate url;
extern crate reqwest;
extern crate semver;

use rocket::response::content;
use rocket_contrib::{Json, Value};
use dotenv::dotenv;

use std::env;
use errors::*;

pub mod auth;
pub mod auth2;
pub mod db;
pub mod errors;
pub mod github;
pub mod models;
pub mod routes;
#[allow(unused_imports)]
pub mod schema;


#[get("/")]
fn index() -> content::Html<&'static str> {
    content::Html(include_str!("../templates/index.html"))
}

#[get("/favicon.ico")]
fn favicon() -> Vec<u8> {
    include_bytes!("../assets/favicon.ico").to_vec()
}

#[get("/assets/style.css")]
fn style() -> content::Css<&'static str> {
    content::Css(include_str!("../assets/style.css"))
}

#[catch(400)]
fn bad_request() -> Json<Value> {
    Json(json!({
        "status": "error",
        "reason": "Bad request"
    }))
}

#[catch(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "status": "error",
        "reason": "Resource was not found"
    }))
}

#[catch(500)]
fn internal_error() -> Json<Value> {
    Json(json!({
        "status": "error",
        "reason": "Internal server error"
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
        .mount("/api/v0", routes![
            routes::api::dashboard,
            routes::api::search,
            routes::api::info,
            routes::api::download,
            routes::api::publish,
            routes::api::login,
            routes::api::authorize,
            routes::api::whoami,
        ])
        .mount("/", routes![
            index,
            favicon,
            style
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
