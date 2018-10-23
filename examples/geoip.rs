extern crate sn0int;
extern crate env_logger;

use std::env;
use sn0int::errors::*;


fn run() -> Result<()> {
    let geoip = sn0int::geoip::GeoIP::open_or_download()?;

    for arg in env::args().skip(1) {
        let ip = arg.parse()?;
        let lookup = geoip.lookup(ip)?;
        println!("{:#?}", lookup);
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
