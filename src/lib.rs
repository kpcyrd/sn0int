#![warn(unused_extern_crates)]
#[macro_use] extern crate failure;
#[macro_use] extern crate maplit;
use url;
use hlua_badtouch as hlua;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate structopt;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate crossbeam_channel as channel;

pub mod api;
pub mod archive;
pub mod args;
pub mod auth;
pub mod autonoscope;
pub mod blobs;
pub mod cmd;
pub mod complete;
pub mod config;
pub mod crt;
pub mod db;
pub mod errors;
pub mod engine;
pub mod filters;
pub mod fmt;
pub mod geoip;
pub mod gfx;
pub mod html;
pub mod json;
pub mod keyring;
pub mod lazy;
pub mod migrations;
pub mod models;
pub mod paths;
pub mod psl;
pub mod options;
pub mod registry;
pub mod runtime;
pub mod sandbox;
pub mod schema;
pub mod ser;
pub mod shell;
pub mod sockets;
pub mod term;
pub mod update;
pub mod utils;
pub mod web;
pub mod worker;
pub mod workspaces;
pub mod xml;

#[cfg(test)]
fn test_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}
