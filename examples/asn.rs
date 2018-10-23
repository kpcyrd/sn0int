extern crate sn0int;
extern crate env_logger;
extern crate maxminddb;

use std::env;
use sn0int::errors::*;
use sn0int::geoip::{AsnDB, Maxmind};


fn run() -> Result<()> {
    let asndb = AsnDB::open_or_download()?;

    for arg in env::args().skip(1) {
        let ip = arg.parse()?;
        let asn = asndb.lookup(ip)?;
        println!("{:#?}", asn);
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
