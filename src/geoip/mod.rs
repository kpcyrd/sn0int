use errors::*;
use archive;
use chrootable_https::Client;
use maxminddb::{self, geoip2};
use std::fmt;
use std::fs::File;
use std::net::IpAddr;
use std::path::Path;
use paths;
use worker;

pub static GEOIP_CITY_URL: &str = "https://geolite.maxmind.com/download/geoip/database/GeoLite2-City.tar.gz";
pub static GEOIP_ASN_URL: &str = "https://geolite.maxmind.com/download/geoip/database/GeoLite2-ASN.tar.gz";

pub mod models;
use self::models::GeoLookup;
use self::models::AsnLookup;


pub trait Maxmind: Sized {
    #[inline]
    fn archive_filename() -> &'static str;

    #[inline]
    fn archive_url() -> &'static str;

    #[inline]
    fn new(reader: maxminddb::Reader) -> Self;

    fn open(path: &str) -> Result<Self> {
        let reader = maxminddb::Reader::open(path)
            .context("Failed to open geoip database")?;
        Ok(Self::new(reader))
    }

    fn open_or_download() -> Result<Self> {
        let path = paths::cache_dir()?.join(Self::archive_filename());

        if File::open(&path).is_err() {
            worker::spawn_fn(&format!("Downloading {:?}", Self::archive_filename()), || {
                Self::download(&path, Self::archive_filename(), Self::archive_url())
            }, false)?;
        };

        let path = path.to_str().ok_or(format_err!("Failed to decode path"))?;
        Self::open(&path)
    }

    fn download<P: AsRef<Path>>(path: P, filter: &str, url: &str) -> Result<()> {
        debug!("Downloading {:?}...", url);
        let client = Client::with_system_resolver()?;
        let resp = client.get(url)?;
        debug!("Downloaded {} bytes", resp.body.len());
        archive::extract(&mut &resp.body[..], filter, path)?;
        Ok(())
    }
}

pub struct GeoIP {
    reader: maxminddb::Reader,
}

impl fmt::Debug for GeoIP {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "GeoIP {{ ... }}")
    }
}

impl Maxmind for GeoIP {
    fn archive_filename() -> &'static str {
        "GeoLite2-City.mmdb"
    }

    fn archive_url() -> &'static str {
        GEOIP_CITY_URL
    }

    fn new(reader: maxminddb::Reader) -> Self {
        GeoIP {
            reader
        }
    }
}

impl GeoIP {
    pub fn lookup(&self, ip: IpAddr) -> Result<GeoLookup> {
        let city: geoip2::City = self.reader.lookup(ip)?;
        debug!("GeoIP result: {:?}", city);
        Ok(GeoLookup::from(city))
    }
}

pub struct AsnDB {
    reader: maxminddb::Reader,
}

impl fmt::Debug for AsnDB {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "GeoIP {{ ... }}")
    }
}

impl Maxmind for AsnDB {
    fn archive_filename() -> &'static str {
        "GeoLite2-ASN.mmdb"
    }

    fn archive_url() -> &'static str {
        GEOIP_ASN_URL
    }

    fn new(reader: maxminddb::Reader) -> Self {
        AsnDB {
            reader
        }
    }
}

impl AsnDB {
    pub fn lookup(&self, ip: IpAddr) -> Result<AsnLookup> {
        let isp: geoip2::Isp = self.reader.lookup(ip)?;
        debug!("ASN result: {:?}", isp);
        AsnLookup::try_from(isp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geoip_lookup() {
        let ip = "1.1.1.1".parse().unwrap();
        let geoip = GeoIP::open_or_download().expect("Failed to load geoip");
        let lookup = geoip.lookup(ip).expect("GeoIP lookup failed");
        println!("{:#?}", lookup);
        assert_eq!(lookup.city, None);
    }

    #[test]
    fn test_asn_lookup() {
        let ip = "1.1.1.1".parse().unwrap();
        let asndb = AsnDB::open_or_download().expect("Failed to load asndb");
        let lookup = asndb.lookup(ip).expect("ASN lookup failed");
        println!("{:#?}", lookup);
    }
}
