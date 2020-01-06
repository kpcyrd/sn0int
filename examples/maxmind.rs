extern crate sn0int;
extern crate env_logger;
extern crate chrootable_https;

// workaround for rustc 1.29.2 support
#[cfg(not(target_os = "openbsd"))]
extern crate structopt;
#[cfg(target_os = "openbsd")]
#[macro_use] extern crate structopt;

use sn0int::errors::*;
use sn0int::geoip::{AsnDB, GeoIP, Maxmind};
use std::net::IpAddr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Args {
    #[structopt(name="asn")]
    Asn(AsnArgs),
    #[structopt(name="geoip")]
    GeoIP(GeoIPArgs),
}

#[derive(Debug, StructOpt)]
pub struct AsnArgs {
    ip: IpAddr,
}

impl AsnArgs {
    fn run(&self) -> Result<()> {
        let path = AsnDB::cache_path()?;
        let asndb = AsnDB::open(&path)?;

        let asn = asndb.lookup(self.ip)?;
        println!("{:#?}", asn);

        Ok(())
    }
}

#[derive(Debug, StructOpt)]
pub struct GeoIPArgs {
    ip: IpAddr,
}

impl GeoIPArgs {
    fn run(&self) -> Result<()> {
        let path = GeoIP::cache_path()?;
        let geoip = GeoIP::open(&path)?;

        let lookup = geoip.lookup(self.ip)?;
        println!("{:#?}", lookup);

        Ok(())
    }
}


fn run() -> Result<()> {
    let args = Args::from_args();
    debug!("{:?}", args);
    match args {
        Args::Asn(args) => args.run(),
        Args::GeoIP(args) => args.run(),
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
