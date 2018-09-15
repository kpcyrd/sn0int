use errors::*;

use chrootable_https::Client;
use std::fs::{self, File};
use std::path::PathBuf;
use paths;
use publicsuffix::{self, Domain, DnsName};
use worker;


pub struct Psl {
    list: publicsuffix::List,
}

impl Psl {
    pub fn open_or_download() -> Result<Psl> {
        let path = paths::cache_dir()?.join("public_suffix_list.dat");

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

    pub fn download(path: &PathBuf, url: &str) -> Result<()> {
        let client = Client::with_system_resolver()?;
        let resp = client.get(url)?;
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
