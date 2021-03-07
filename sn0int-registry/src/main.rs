#![allow(proc_macro_derive_resolution_fallback)]
#![warn(unused_extern_crates)]
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate rocket_failure;

use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::http::Header;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::templates::Template;
use sn0int_registry::db;
use sn0int_registry::errors::*;
use std::env;
use structopt::StructOpt;

pub mod assets;
pub mod auth;
pub mod auth2;
pub mod github;
pub mod routes;


#[catch(400)]
fn bad_request() -> Json<JsonValue> {
    Json(json!({
        "error": "Bad request"
    }))
}

#[catch(404)]
fn not_found() -> Json<JsonValue> {
    Json(json!({
        "error": "Resource was not found"
    }))
}

#[catch(500)]
fn internal_error() -> Json<JsonValue> {
    Json(json!({
        "error": "Internal server error"
    }))
}

#[derive(Debug, StructOpt)]
pub struct Args {
}

fn run() -> Result<()> {
    let _args = Args::from_args();

    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL must be set")?;

    db::setup_db(&database_url, 60)
        .context("Failed to setup db")?;

    rocket::ignite()
        .manage(db::init(&database_url))
        .attach(Template::fairing())
        .attach(AdHoc::on_response("Security Headers", |_, resp| {
            resp.set_header(Header::new("Strict-Transport-Security", "max-age=63072000; includeSubDomains; preload"));
            resp.set_header(Header::new("Content-Security-Policy", "style-src 'self'"));
            resp.set_header(Header::new("Feature-Policy", "geolocation 'none'; midi 'none'; notifications 'none'; push 'none'; sync-xhr 'none'; microphone 'none'; camera 'none'; magnetometer 'none'; gyroscope 'none'; speaker 'none'; vibrate 'none'; fullscreen 'none'; payment 'none'"));
            resp.set_header(Header::new("X-Frame-Options", "deny"));
            resp.set_header(Header::new("X-XSS-Protection", "1; mode=block"));
            resp.set_header(Header::new("X-Content-Type-Options", "nosniff"));
            resp.set_header(Header::new("Referrer-Policy", "same-origin"));
        }))
        .mount("/api/v0", routes![
            routes::api::quickstart,
            routes::api::latest,
            routes::api::search,
            routes::api::info,
            routes::api::download,
            routes::api::publish,
            routes::api::whoami,
        ])
        .mount("/api/v0/badges", routes![
            routes::badges::modules,
            routes::badges::downloads,
        ])
        .mount("/auth", routes![
            routes::auth::get,
            routes::auth::post,
            routes::auth::login,
        ])
        .mount("/", routes![
            routes::html::index,
            routes::html::details,
            routes::assets::favicon,
            routes::assets::style,
            routes::assets::javascript,
            routes::assets::social_card,
            routes::assets::javascript_clipboard,
            routes::assets::fontawesome_style,
            routes::assets::fontawesome_font,
            routes::health::health,
        ])
        .register(catchers![
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
