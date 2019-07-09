use crate::errors::*;

use chrootable_https::Client;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::paths;
use crate::lazy::LazyInit;
use publicsuffix::{self, Domain, DnsName};
use crate::worker;


#[derive(Debug)]
pub enum PslReader {
    Reader(File),
    String(String),
}

impl PslReader {
    pub fn open_or_download() -> Result<PslReader> {
        let reader = match Self::open() {
            Ok(r) => r,
            Err(_) => worker::spawn_fn("Downloading public suffix list", || {
                PslReader::download(&Self::path()?, publicsuffix::LIST_URL)?;
                Self::open()
            }, false)?,
        };
        Ok(reader)
    }

    pub fn open() -> Result<PslReader> {
        let file = File::open(Self::path()?)?;
        Ok(PslReader::Reader(file))
    }

    pub fn path() -> Result<PathBuf> {
        // use system path if exists
        let path = Path::new("/usr/share/publicsuffix/public_suffix_list.dat");
        if path.exists() {
            return Ok(path.to_path_buf());
        }

        // else, use local cache
        let path = paths::cache_dir()?
            .join("public_suffix_list.dat");
        Ok(path)
    }

    pub fn download(path: &Path, url: &str) -> Result<()> {
        let client = Client::with_system_resolver()?;
        let resp = client.get(url)
            .wait_for_response()
            .context("http request failed")?;
        fs::write(path, &resp.body)?;
        Ok(())
    }
}

impl LazyInit<Arc<Psl>> for PslReader {
    fn initialize(self) -> Result<Arc<Psl>> {
        let list = match self {
            PslReader::Reader(file) => publicsuffix::List::from_reader(file),
            PslReader::String(s) => publicsuffix::List::from_str(&s),
        };

        let list = list
            .map_err(|e| format_err!("Failed to load public suffix list: {}", e))?;

        Ok(Arc::new(Psl {
            list,
        }))
    }
}

#[derive(Debug)]
pub struct Psl {
    list: publicsuffix::List,
}

impl Psl {
    pub fn parse_domain(&self, domain: &str) -> Result<Domain> {
        self.list.parse_domain(domain)
            .map_err(|e| format_err!("Failed to parse domain: {}", e))
    }

    pub fn parse_dns_name(&self, name: &str) -> Result<DnsName> {
        self.list.parse_dns_name(name)
            .map_err(|e| format_err!("Failed to parse dns_name: {}", e))
    }
}
