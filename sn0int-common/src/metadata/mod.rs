use crate::errors::*;
use nom::bytes::complete::{tag, take_until};
use nom::combinator::map_res;
use nom::multi::fold_many0;
use nom::IResult;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

mod stealth;
pub use self::stealth::Stealth;

#[derive(Debug, PartialEq, Clone)]
pub enum EntryType {
    Description,
    Version,
    Source,
    KeyringAccess,
    Stealth,
    Author,
    Repository,
    License,
}

impl FromStr for EntryType {
    type Err = Error;

    fn from_str(s: &str) -> Result<EntryType> {
        match s {
            "Description" => Ok(EntryType::Description),
            "Version" => Ok(EntryType::Version),
            "Source" => Ok(EntryType::Source),
            "Keyring-Access" => Ok(EntryType::KeyringAccess),
            "Stealth" => Ok(EntryType::Stealth),
            "Author" => Ok(EntryType::Author),
            "Repository" => Ok(EntryType::Repository),
            "License" => Ok(EntryType::License),
            x => bail!("Unknown EntryType: {:?}", x),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Source {
    Domains,
    Subdomains,
    IpAddrs,
    Urls,
    Emails,
    PhoneNumbers,
    Networks,
    Devices,
    Accounts(Option<String>),
    Breaches,
    Images,
    Ports,
    Netblocks,
    CryptoAddrs(Option<String>),
    KeyRing(String),
    Notifications,
}

impl Source {
    pub fn group_as_str(&self) -> &'static str {
        match self {
            Source::Domains => "domains",
            Source::Subdomains => "subdomains",
            Source::IpAddrs => "ipaddrs",
            Source::Urls => "urls",
            Source::Emails => "emails",
            Source::PhoneNumbers => "phonenumbers",
            Source::Networks => "networks",
            Source::Devices => "devices",
            Source::Accounts(_) => "accounts",
            Source::Breaches => "breaches",
            Source::Images => "images",
            Source::Ports => "ports",
            Source::Netblocks => "netblocks",
            Source::CryptoAddrs(_) => "cryptoaddrs",
            Source::Notifications => "notifications",
            Source::KeyRing(_) => "keyring",
        }
    }
}

impl FromStr for Source {
    type Err = Error;

    fn from_str(s: &str) -> Result<Source> {
        let (key, param) = if let Some(idx) = s.find(':') {
            let (a, b) = s.split_at(idx);
            (a, Some(&b[1..]))
        } else {
            (s, None)
        };

        match (key, param) {
            ("domains", None) => Ok(Source::Domains),
            ("subdomains", None) => Ok(Source::Subdomains),
            ("ipaddrs", None) => Ok(Source::IpAddrs),
            ("urls", None) => Ok(Source::Urls),
            ("emails", None) => Ok(Source::Emails),
            ("phonenumbers", None) => Ok(Source::PhoneNumbers),
            ("networks", None) => Ok(Source::Networks),
            ("devices", None) => Ok(Source::Devices),
            ("accounts", param) => Ok(Source::Accounts(param.map(String::from))),
            ("breaches", None) => Ok(Source::Breaches),
            ("images", None) => Ok(Source::Images),
            ("ports", None) => Ok(Source::Ports),
            ("netblocks", None) => Ok(Source::Netblocks),
            ("cryptoaddrs", param) => Ok(Source::CryptoAddrs(param.map(String::from))),
            ("notifications", None) => Ok(Source::Notifications),
            ("keyring", Some(param)) => Ok(Source::KeyRing(param.to_string())),
            (x, Some(param)) => bail!("Unknown Source: {:?} ({:?})", x, param),
            (x, None) => bail!("Unknown Source: {:?}", x),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum License {
    MIT,
    GPL3,
    LGPL3,
    BSD2,
    BSD3,
    WTFPL,
}

impl FromStr for License {
    type Err = Error;

    fn from_str(s: &str) -> Result<License> {
        match s {
            "MIT" => Ok(License::MIT),
            "GPL-3.0" => Ok(License::GPL3),
            "LGPL-3.0" => Ok(License::LGPL3),
            "BSD-2-Clause" => Ok(License::BSD2),
            "BSD-3-Clause" => Ok(License::BSD3),
            "WTFPL" => Ok(License::WTFPL),
            x => bail!("Unsupported license: {:?}", x),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Metadata {
    pub description: String,
    pub version: String,
    pub source: Option<Source>,
    pub keyring_access: Vec<String>,
    pub stealth: Stealth,
    pub authors: Vec<String>,
    pub repository: Option<String>,
    pub license: License,
}

impl FromStr for Metadata {
    type Err = Error;

    fn from_str(code: &str) -> Result<Metadata> {
        let (_, lines) = metalines(code)
            .map_err(|_| format_err!("Failed to parse header"))?;

        let mut data = NewMetadata::default();

        for (k, v) in lines {
            match k {
                EntryType::Description => data.description = Some(v),
                EntryType::Version => data.version = Some(v),
                EntryType::Source => data.source = Some(v),
                EntryType::KeyringAccess => data.keyring_access.push(v),
                EntryType::Stealth => data.stealth = Some(v),
                EntryType::Author => data.authors.push(v),
                EntryType::Repository => data.repository = Some(v),
                EntryType::License => data.license = Some(v),
            }
        }

        data.try_from()
    }
}

#[derive(Default)]
pub struct NewMetadata<'a> {
    pub description: Option<&'a str>,
    pub version: Option<&'a str>,
    pub source: Option<&'a str>,
    pub keyring_access: Vec<&'a str>,
    pub stealth: Option<&'a str>,
    pub authors: Vec<&'a str>,
    pub repository: Option<&'a str>,
    pub license: Option<&'a str>,
}

impl<'a> NewMetadata<'a> {
    fn try_from(self) -> Result<Metadata> {
        let description = self.description.ok_or_else(|| format_err!("Description is required"))?;
        let version = self.version.ok_or_else(|| format_err!("Version is required"))?;
        let source = match self.source {
            Some(x) => Some(x.parse()?),
            _ => None,
        };
        let keyring_access = self.keyring_access.into_iter()
            .map(String::from)
            .collect();
        let stealth = match self.stealth {
            Some(x) => x.parse()?,
            _ => Stealth::Normal,
        };
        let authors = self.authors.into_iter()
            .map(String::from)
            .collect();
        let repository = self.repository.map(String::from);
        let license = self.license.ok_or_else(|| format_err!("License is required"))?;
        let license = license.parse()?;

        Ok(Metadata {
            description: description.to_string(),
            version: version.to_string(),
            source,
            keyring_access,
            stealth,
            authors,
            repository,
            license,
        })
    }
}

fn metaline(input: &str) -> IResult<&str, (EntryType, &str)> {
    let (input, _) = tag("-- ")(input)?;
    let (input, name) = map_res(
        take_until(": "),
        EntryType::from_str
    )(input)?;
    let (input, _) = tag(": ")(input)?;
    let (input, value) = take_until("\n")(input)?;
    let (input, _) = tag("\n")(input)?;

    Ok((input, (name, value)))
}

fn metalines(input: &str) -> IResult<&str, Vec<(EntryType, &str)>> {
    let (input, lines) = fold_many0(
        metaline,
        Vec::new,
        |mut acc: Vec<_>, item| {
            acc.push(item);
            acc
        }
    )(input)?;
    let (input, _) = tag("\n")(input)?;

    Ok((input, lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_simple() {
        let metadata = Metadata::from_str(r#"-- Description: Hello world, this is my description
-- Version: 1.0.0
-- Source: domains
-- License: WTFPL

"#).expect("parse");
        assert_eq!(metadata, Metadata {
            description: "Hello world, this is my description".to_string(),
            version: "1.0.0".to_string(),
            license: License::WTFPL,
            source: Some(Source::Domains),
            stealth: Stealth::Normal,
            authors: vec![],
            repository: None,
            keyring_access: Vec::new(),
        });
    }

    #[test]
    fn verify_much_metadata() {
        let metadata = Metadata::from_str(r#"-- Description: Hello world, this is my description
-- Version: 1.0.0
-- Source: domains
-- Stealth: passive
-- Author: kpcyrd <git at rxv dot cc>
-- Author: kpcyrd's cat
-- Repository: https://github.com/kpcyrd/sn0int
-- License: WTFPL

"#).expect("parse");
        assert_eq!(metadata, Metadata {
            description: "Hello world, this is my description".to_string(),
            version: "1.0.0".to_string(),
            license: License::WTFPL,
            source: Some(Source::Domains),
            stealth: Stealth::Passive,
            authors: vec![
                "kpcyrd <git at rxv dot cc>".to_string(),
                "kpcyrd's cat".to_string(),
            ],
            repository: Some("https://github.com/kpcyrd/sn0int".to_string()),
            keyring_access: Vec::new(),
        });
    }

    #[test]
    fn verify_no_source() {
        let metadata = Metadata::from_str(r#"-- Description: Hello world, this is my description
-- Version: 1.0.0
-- License: WTFPL

"#).expect("parse");
        assert_eq!(metadata, Metadata {
            description: "Hello world, this is my description".to_string(),
            version: "1.0.0".to_string(),
            license: License::WTFPL,
            source: None,
            stealth: Stealth::Normal,
            authors: vec![],
            repository: None,
            keyring_access: Vec::new(),
        });
    }

    #[test]
    fn verify_require_license() {
        let metadata = Metadata::from_str(r#"-- Description: Hello world, this is my description
-- Version: 1.0.0
-- Source: domains

"#);
        assert!(metadata.is_err());
    }

    #[test]
    fn verify_require_opensource_license() {
        let metadata = Metadata::from_str(r#"-- Description: Hello world, this is my description
-- Version: 1.0.0
-- Source: domains
-- License: Proprietary

"#);
        assert!(metadata.is_err());
    }

    #[test]
    fn verify_keyring_source() {
        let x = Source::from_str("keyring:foo").unwrap();
        assert_eq!(x, Source::KeyRing("foo".to_string()));

        let x = Source::from_str("keyring:").unwrap();
        assert_eq!(x, Source::KeyRing("".to_string()));
    }

    #[test]
    fn verify_invalid_keyring_source() {
        let x = Source::from_str("keyring");
        assert!(x.is_err());
    }

    #[test]
    fn verify_account_source() {
        let x = Source::from_str("accounts").unwrap();
        assert_eq!(x, Source::Accounts(None));

        let x = Source::from_str("accounts:github.com").unwrap();
        assert_eq!(x, Source::Accounts(Some("github.com".into())));
    }
}
