extern crate sn0int;
extern crate env_logger;
extern crate chrootable_https;
#[macro_use] extern crate log;

// workaround for rustc 1.29.2 support
#[cfg(not(target_os = "openbsd"))]
extern crate structopt;
#[cfg(target_os = "openbsd")]
#[macro_use] extern crate structopt;

use sn0int::errors::*;
use sn0int::geoip::{GeoIP, Maxmind};
use sn0int::paths;
use std::fs;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    url: String,
    filter: String,
    target: String,
    #[structopt(short="e", long="extract-only")]
    extract_only: bool,
}

fn run() -> Result<()> {
    let args = Args::from_args();
    debug!("{:?}", args);
    let path = paths::cache_dir()?.join(&args.target);
    if args.extract_only {
        let body = fs::read(&args.url)?;
        sn0int::archive::extract(&mut &body[..], &args.filter, path)?;
    } else {
        GeoIP::download(path, &args.filter, &args.url)?;
    }
    Ok(())
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
