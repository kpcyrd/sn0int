extern crate sn0int;
extern crate env_logger;
extern crate chrootable_https;
#[macro_use] extern crate log;
#[macro_use] extern crate structopt;

use sn0int::errors::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    url: String,
    filter: String,
    target: String,
}

fn run() -> Result<()> {
    let args = Args::from_args();
    debug!("{:?}", args);
    sn0int::geoip::GeoIP::download(&args.target, &args.filter, &args.url)?;
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
