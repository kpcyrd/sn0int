use crate::errors::*;
use crate::lazy::LazyInit;
use maxminddb::{self, geoip2};
use std::fmt;
use std::fs::{self, File};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::sync::Arc;

pub mod models;
use self::models::GeoLookup;
use self::models::AsnLookup;


pub trait Maxmind: Sized {
    fn filename() -> &'static str;

    fn new(reader: maxminddb::Reader<Vec<u8>>) -> Self;

    fn cache_path(cache_dir: &Path) -> Result<PathBuf> {
        // use system path if exists
        for path in &[
            // Archlinux
            "/usr/share/GeoIP/",
            // OpenBSD
            "/usr/local/share/examples/libmaxminddb/",
            // geoipupdate
            "/var/lib/GeoIP/",
        ] {
            let path = Path::new(path);
            let path = path.join(Self::filename());

            if path.exists() {
                return Ok(path);
            }
        }

        // use cache path
        let path = cache_dir
            .join(Self::filename());
        Ok(path)
    }

    fn from_buf(buf: Vec<u8>) -> Result<Self> {
        let reader = maxminddb::Reader::from_source(buf)
            .context("Failed to read geoip database")?;
        Ok(Self::new(reader))
    }

    fn open(path: &Path) -> Result<Self> {
        let buf = fs::read(path)?;
        Self::from_buf(buf)
    }

    fn try_open_reader(cache_dir: &Path) -> Result<Option<MaxmindReader>> {
        let path = Self::cache_path(cache_dir)?;

        if path.exists() {
            let db = MaxmindReader::open_path(path)?;
            Ok(Some(db))
        } else {
            Ok(None)
        }
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
    fn filename() -> &'static str {
        "GeoLite2-City.mmdb"
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
    fn filename() -> &'static str {
        "GeoLite2-ASN.mmdb"
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
    // You need geoip setup on your system to run this
    /*
    use super::*;

    #[test]
    #[ignore]
    fn test_geoip_lookup() {
        let ip = "1.1.1.1".parse().unwrap();
        let path = GeoIP::cache_path().unwrap();
        let geoip = GeoIP::open(&path).unwrap();
        let lookup = geoip.lookup(ip).expect("GeoIP lookup failed");
        println!("{:#?}", lookup);
        assert_eq!(lookup.city, None);
    }

    #[test]
    #[ignore]
    fn test_asn_lookup() {
        let ip = "1.1.1.1".parse().unwrap();
        let path = AsnDB::cache_path().unwrap();
        let asndb = AsnDB::open(&path).unwrap();
        let lookup = asndb.lookup(ip).expect("ASN lookup failed");
        println!("{:#?}", lookup);
    }
    */
}
