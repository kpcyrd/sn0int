use chrootable_https::Client;
use crate::archive;
use crate::errors::*;
use crate::lazy::LazyInit;
use crate::paths;
use crate::worker;
use maxminddb::{self, geoip2};
use std::fmt;
use std::fs::{self, File};
use std::net::IpAddr;
use std::path::Path;
use std::io::Read;
use std::sync::Arc;

pub static GEOIP_CITY_URL: &str = "https://geolite.maxmind.com/download/geoip/database/GeoLite2-City.tar.gz";
pub static GEOIP_ASN_URL: &str = "https://geolite.maxmind.com/download/geoip/database/GeoLite2-ASN.tar.gz";

pub mod models;
use self::models::GeoLookup;
use self::models::AsnLookup;


pub trait Maxmind: Sized {
    fn archive_filename() -> &'static str;

    fn archive_url() -> &'static str;

    fn new(reader: maxminddb::Reader<Vec<u8>>) -> Self;

    // TODO: refactor this to return Path
    fn cache_path() -> Result<String> {
        // use system path if exists
        for path in &[
            // Archlinux
            "/usr/share/GeoIP/",
            // OpenBSD
            "/usr/local/share/examples/libmaxminddb/",
        ] {
            let path = Path::new(path);
            let path = path.join(Self::archive_filename());

            if path.exists() {
                let path = path.to_str()
                    .ok_or_else(|| format_err!("Failed to decode path"))?;
                return Ok(path.to_string());
            }
        }

        // use cache path
        let path = paths::cache_dir()?
            .join(Self::archive_filename());
        let path = path.to_str()
            .ok_or_else(|| format_err!("Failed to decode path"))?;
        Ok(path.to_string())
    }

    fn from_buf(buf: Vec<u8>) -> Result<Self> {
        let reader = maxminddb::Reader::from_source(buf)
            .context("Failed to read geoip database")?;
        Ok(Self::new(reader))
    }

    fn open(path: &str) -> Result<Self> {
        let buf = fs::read(path)?;
        Self::from_buf(buf)
    }

    fn open_reader() -> Result<MaxmindReader> {
        let path = Self::cache_path()?;
        MaxmindReader::open_path(path)
    }

    fn open_or_download() -> Result<Self> {
        let path = Self::cache_path()?;

        if File::open(&path).is_err() {
            worker::spawn_fn(&format!("Downloading {:?}", Self::archive_filename()), || {
                Self::download(&path, Self::archive_filename(), Self::archive_url())
            }, false)?;
        };

        Self::open(&path)
    }

    fn download<P: AsRef<Path>>(path: P, filter: &str, url: &str) -> Result<()> {
        debug!("Downloading {:?}...", url);
        let client = Client::with_system_resolver()?;
        let resp = client.get(url)
            .wait_for_response()
            .context("http request failed")?;
        debug!("Downloaded {} bytes", resp.body.len());
        archive::extract(&mut &resp.body[..], filter, path)?;
        Ok(())
    }
}

pub struct MaxmindReader {
    reader: File,
}

impl MaxmindReader {
    fn open_path<P: AsRef<Path>>(path: P) -> Result<MaxmindReader> {
        let reader = File::open(path)?;
        Ok(MaxmindReader {
            reader,
        })
    }
}

impl fmt::Debug for MaxmindReader {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "MaxmindReader {{ ... }}")
    }
}

impl LazyInit<Arc<GeoIP>> for MaxmindReader {
    fn initialize(mut self) -> Result<Arc<GeoIP>> {
        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf)?;
        Ok(Arc::new(GeoIP::from_buf(buf)?))
    }
}

impl LazyInit<Arc<AsnDB>> for MaxmindReader {
    fn initialize(mut self) -> Result<Arc<AsnDB>> {
        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf)?;
        Ok(Arc::new(AsnDB::from_buf(buf)?))
    }
}

pub struct GeoIP {
    reader: maxminddb::Reader<Vec<u8>>,
}

impl fmt::Debug for GeoIP {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "GeoIP {{ ... }}")
    }
}

impl Maxmind for GeoIP {
    #[inline]
    fn archive_filename() -> &'static str {
        "GeoLite2-City.mmdb"
    }

    #[inline]
    fn archive_url() -> &'static str {
        GEOIP_CITY_URL
    }

    #[inline]
    fn new(reader: maxminddb::Reader<Vec<u8>>) -> Self {
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
    reader: maxminddb::Reader<Vec<u8>>,
}

impl fmt::Debug for AsnDB {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "GeoIP {{ ... }}")
    }
}

impl Maxmind for AsnDB {
    #[inline]
    fn archive_filename() -> &'static str {
        "GeoLite2-ASN.mmdb"
    }

    #[inline]
    fn archive_url() -> &'static str {
        GEOIP_ASN_URL
    }

    #[inline]
    fn new(reader: maxminddb::Reader<Vec<u8>>) -> Self {
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
