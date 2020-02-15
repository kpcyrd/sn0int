use sn0int::errors::*;
use sn0int::geoip::{AsnDB, GeoIP, Maxmind};
use sn0int::paths;
use std::net::IpAddr;
use std::path::Path;
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
    fn run(&self, cache_dir: &Path) -> Result<()> {
        let path = AsnDB::cache_path(cache_dir)?;
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
    fn run(&self, cache_dir: &Path) -> Result<()> {
        let path = GeoIP::cache_path(cache_dir)?;
        let geoip = GeoIP::open(&path)?;

        let lookup = geoip.lookup(self.ip)?;
        println!("{:#?}", lookup);

        Ok(())
    }
}


fn run() -> Result<()> {
    let args = Args::from_args();
    debug!("{:?}", args);
    let cache_dir = paths::cache_dir()?;
    match args {
        Args::Asn(args) => args.run(&cache_dir),
        Args::GeoIP(args) => args.run(&cache_dir),
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
