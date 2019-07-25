use crate::errors::*;

use chrootable_https::Client;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::paths;
use crate::lazy::LazyInit;
use crate::worker;


#[derive(Debug, PartialEq)]
pub struct DnsName {
    pub fulldomain: Option<String>,
    pub root: String,
    pub suffix: String,
}

#[derive(Debug)]
pub enum PslReader {
    Reader(File),
    String(String),
}

impl PslReader {
    pub fn open_or_download() -> Result<PslReader> {
        let path = Self::path()?;
        let reader = match Self::open_from(&path) {
            Ok(r) => r,
            Err(_) => worker::spawn_fn("Downloading public suffix list", || {
                PslReader::download(&path, publicsuffix::LIST_URL)?;
                Self::open_from(&path)
            }, false)?,
        };
        Ok(reader)
    }

    pub fn open() -> Result<PslReader> {
        let path = Self::path()?;
        Self::open_from(&path)

    }

    pub fn open_from(path: &Path) -> Result<PslReader> {
        let file = File::open(path)?;
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
    pub fn parse_dns_name(&self, name: &str) -> Result<DnsName> {
        let dns_name = self.list.parse_dns_name(name)
            .map_err(|e| format_err!("Failed to parse dns_name: {}", e))?;

        let (root, suffix) = if let Some(domain) = dns_name.domain() {
            let root = domain.to_string();
            if let Some(suffix) = domain.suffix() {
                (root, suffix.to_string())
            } else {
                // XXX: not sure if this is reachable
                (root.clone(), root)
            }
        } else {
            // this is technically a tld, but support eg. a.prod.fastly.net anyway
            // XXX: consider forcing domain to be in self.list.private()
            (name.to_string(), name.to_string())
        };

        let fulldomain = if root.as_str() != name {
            Some(name.to_string())
        } else {
            None
        };

        Ok(DnsName {
            fulldomain,
            root,
            suffix,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn init() -> Arc<Psl> {
        PslReader::String(r#"
// ===BEGIN ICANN DOMAINS===
com
// ===END ICANN DOMAINS===
// ===BEGIN PRIVATE DOMAINS===
a.prod.fastly.net
// ===END PRIVATE DOMAINS===
"#.into()).initialize().unwrap()
    }

    #[test]
    fn test_psl_example_com() {
        let x = init().parse_dns_name("example.com").expect("parse_dns_name");
        assert_eq!(x, DnsName {
            fulldomain: None,
            root: "example.com".into(),
            suffix: "com".into(),
        });
    }

    #[test]
    fn test_psl_www_example_com() {
        let x = init().parse_dns_name("www.example.com").expect("parse_dns_name");
        assert_eq!(x, DnsName {
            fulldomain: Some("www.example.com".into()),
            root: "example.com".into(),
            suffix: "com".into(),
        });
    }

    #[test]
    fn test_psl_com() {
        let x = init().parse_dns_name("com").expect("parse_dns_name");
        assert_eq!(x, DnsName {
            fulldomain: None,
            root: "com".into(),
            suffix: "com".into(),
        });
    }

    #[test]
    fn test_psl_a_b_c_d_e_f_g_com() {
        let x = init().parse_dns_name("a.b.c.d.e.f.g.com").expect("parse_dns_name");
        assert_eq!(x, DnsName {
            fulldomain: Some("a.b.c.d.e.f.g.com".into()),
            root: "g.com".into(),
            suffix: "com".into(),
        });
    }

    #[test]
    fn test_psl_empty() {
        let x = init().parse_dns_name("").is_err();
        assert!(x);
    }

    #[test]
    fn test_psl_asdfinvalid() {
        let x = init().parse_dns_name("asdfinvalid").expect("parse_dns_name");
        assert_eq!(x, DnsName {
            fulldomain: None,
            root: "asdfinvalid".into(),
            suffix: "asdfinvalid".into(),
        });
    }

    #[test]
    fn test_psl_www_example_asdfinvalid() {
        let x = init().parse_dns_name("www.example.asdfinvalid").expect("parse_dns_name");
        assert_eq!(x, DnsName {
            fulldomain: Some("www.example.asdfinvalid".into()),
            root: "example.asdfinvalid".into(),
            suffix: "asdfinvalid".into(),
        });
    }

    #[test]
    fn test_psl_a_prod_fastly_net() {
        let x = init().parse_dns_name("a.prod.fastly.net").expect("parse_dns_name");
        assert_eq!(x, DnsName {
            fulldomain: None,
            root: "a.prod.fastly.net".into(),
            suffix: "a.prod.fastly.net".into(),
        });
    }

    #[test]
    fn test_psl_www_a_prod_fastly_net() {
        let x = init().parse_dns_name("www.a.prod.fastly.net").expect("parse_dns_name");
        assert_eq!(x, DnsName {
            fulldomain: None,
            root: "www.a.prod.fastly.net".into(),
            suffix: "a.prod.fastly.net".into(),
        });
    }
}
