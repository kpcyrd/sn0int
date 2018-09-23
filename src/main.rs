#![allow(proc_macro_derive_resolution_fallback)]
extern crate rustyline;
extern crate env_logger;
extern crate rand;
extern crate colored;
#[macro_use] extern crate failure;
extern crate shellwords;
extern crate dirs;
extern crate publicsuffix;
extern crate chrootable_https;
extern crate trust_dns_proto;
extern crate url;
extern crate hlua_badtouch as hlua;
extern crate hyper;
extern crate base64;
extern crate http;
extern crate kuchiki;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate nom;
#[macro_use] extern crate structopt;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate crossbeam_channel as channel;

pub mod args;
pub mod cmd;
pub mod complete;
pub mod db;
pub mod errors;
pub mod engine;
pub mod html;
pub mod json;
pub mod migrations;
pub mod models;
pub mod paths;
pub mod ser;
pub mod schema;
pub mod shell;
pub mod runtime;
pub mod term;
pub mod web;
pub mod worker;
pub mod psl;
pub mod utils;

use args::{Args, SubCommand};
use errors::*;
use engine::Module;
use structopt::StructOpt;
use std::path::Path;


fn run_run(gargs: &args::Args, args: &args::Run) -> Result<()> {
    let mut rl = shell::init(gargs)?;

    if let Some(module) = &args.module {
        let module = rl.engine().get(&module)?.clone();
        rl.set_module(module);
    } else if let Some(file) = &args.file {
        let path = Path::new(file);

        let filename = path.file_stem()
            .ok_or(format_err!("Failed to decode filename"))?
            .to_str()
            .ok_or(format_err!("Failed to decode filename"))?;

        let module = Module::load(&path.to_path_buf(), "anonymous", &filename)
            .context(format!("Failed to parse {:?}", file))?;
        rl.set_module(module);
    } else {
        bail!("At least one module or file need to be provided");
    }

    cmd::run_cmd::execute(&mut rl)
}

fn run() -> Result<()> {
    let args = Args::from_args();
    match args.subcommand {
        Some(SubCommand::Run(ref run)) => run_run(&args, run),
        Some(SubCommand::Sandbox(_)) => engine::isolation::run_worker(),
        None => shell::run(&args),
    }
}

fn main() {
    env_logger::init();

    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        for cause in err.iter_chain().skip(1) {
            eprintln!("Because: {}", cause);
        }
        std::process::exit(1);
    }
}
