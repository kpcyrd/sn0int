use crate::errors::*;

use chrootable_https::Client;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::paths;
use publicsuffix::{self, Domain, DnsName};
use crate::worker;


#[derive(Debug)]
pub struct Psl {
    list: publicsuffix::List,
}

impl Psl {
    pub fn open_or_download() -> Result<Psl> {
        let path = Self::path()?;

        let reader = match File::open(&path) {
            Ok(f) => f,
            Err(_) => worker::spawn_fn("Downloading public suffix list", || {
                Psl::download(&path, publicsuffix::LIST_URL)?;

                let f = File::open(&path)?;
                Ok(f)
            }, false)?,
        };

        let list = publicsuffix::List::from_reader(reader)
            .map_err(|e| format_err!("Failed to load public suffix list: {}", e))?;

        Ok(Psl {
            list,
        })
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

    pub fn open_into_string() -> Result<String> {
        let path = Self::path()?;
        let s = fs::read_to_string(&path)?;
        Ok(s)
    }

    pub fn download(path: &Path, url: &str) -> Result<()> {
        let client = Client::with_system_resolver()?;
        let resp = client.get(url)
            .wait_for_response()
            .context("http request failed")?;
        fs::write(path, &resp.body)?;
        Ok(())
    }

    pub fn parse_domain(&self, domain: &str) -> Result<Domain> {
        self.list.parse_domain(domain)
            .map_err(|e| format_err!("Failed to parse domain: {}", e))
    }

    pub fn parse_dns_name(&self, name: &str) -> Result<DnsName> {
        self.list.parse_dns_name(name)
            .map_err(|e| format_err!("Failed to parse dns_name: {}", e))
    }
}

impl FromStr for Psl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Psl> {
        let list = publicsuffix::List::from_str(s)
            .map_err(|e| format_err!("Failed to load public suffix list: {}", e))?;

        Ok(Psl {
            list,
        })
    }
}
