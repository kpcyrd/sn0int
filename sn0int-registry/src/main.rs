#![warn(unused_extern_crates)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate dotenv;
extern crate diesel;

use rocket::response::content;
use dotenv::dotenv;

use std::env;

pub mod db;
pub mod models;
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

fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    rocket::ignite()
        .manage(db::init(&database_url))
        .mount("/", routes![
            index,
            favicon,
            style
        ])
    .launch();
}
