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
use sn0int::geoip::{AsnDB, GeoIP, Maxmind};
use sn0int::paths;
use std::fs;
use std::net::IpAddr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Args {
    #[structopt(name="dl")]
    Download(Download),
    #[structopt(name="asn")]
    Asn(AsnArgs),
    #[structopt(name="geoip")]
    GeoIP(GeoIPArgs),
}

#[derive(Debug, StructOpt)]
pub struct Download {
    url: String,
    filter: String,
    target: String,
    #[structopt(short="e", long="extract-only")]
    extract_only: bool,
}

impl Download {
    fn run(&self) -> Result<()> {
        let path = paths::cache_dir()?.join(&self.target);
        if self.extract_only {
            let body = fs::read(&self.url)?;
            sn0int::archive::extract(&mut &body[..], &self.filter, path)?;
        } else {
            GeoIP::download(path, &self.filter, &self.url)?;
        }
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
pub struct AsnArgs {
    ip: IpAddr,
}

impl AsnArgs {
    fn run(&self) -> Result<()> {
        let asndb = AsnDB::open_or_download()?;

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
        let geoip = GeoIP::open_or_download()?;

        let lookup = geoip.lookup(self.ip)?;
        println!("{:#?}", lookup);

        Ok(())
    }
}


fn run() -> Result<()> {
    let args = Args::from_args();
    debug!("{:?}", args);
    match args {
        Args::Download(args) => args.run(),
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
