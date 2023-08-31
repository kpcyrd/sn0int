use crate::errors::*;
use crate::db;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum Target {
    /// On domains
    #[command(name="domains")]
    Domains(Filter),
    /// On subdomains
    #[command(name="subdomains")]
    Subdomains(Filter),
    /// On ipaddrs
    #[command(name="ipaddrs")]
    IpAddrs(Filter),
    /// On urls
    #[command(name="urls")]
    Urls(Filter),
    /// On emails
    #[command(name="emails")]
    Emails(Filter),
    /// On phone numbers
    #[command(name="phonenumbers")]
    PhoneNumbers(Filter),
    /// On devices
    #[command(name="devices")]
    Devices(Filter),
    /// On networks
    #[command(name="networks")]
    Networks(Filter),
    /// On accounts
    #[command(name="accounts")]
    Accounts(Filter),
    /// On breaches
    #[command(name="breaches")]
    Breaches(Filter),
    /// On images
    #[command(name="images")]
    Images(Filter),
    /// On ports
    #[command(name="ports")]
    Ports(Filter),
    /// On ipnets
    #[command(name="netblocks")]
    Netblocks(Filter),
    /// On crypto currency addresses
    #[command(name="cryptoaddrs")]
    CryptoAddrs(Filter),
}

impl Target {
    pub fn domains(&self) -> Option<&Filter> {
        if let Target::Domains(f) = self {
            Some(f)
        } else {
            None
        }
    }

    pub fn subdomains(&self) -> Option<&Filter> {
        if let Target::Subdomains(f) = self {
            Some(f)
        } else {
            None
        }
    }

    pub fn ipaddrs(&self) -> Option<&Filter> {
        if let Target::IpAddrs(f) = self {
            Some(f)
        } else {
            None
        }
    }

    pub fn urls(&self) -> Option<&Filter> {
        if let Target::Urls(f) = self {
            Some(f)
        } else {
            None
        }
    }

    pub fn ports(&self) -> Option<&Filter> {
        if let Target::Ports(f) = self {
            Some(f)
        } else {
            None
        }
    }

    pub fn netblocks(&self) -> Option<&Filter> {
        if let Target::Netblocks(f) = self {
            Some(f)
        } else {
            None
        }
    }
}

#[derive(Debug, Parser)]
pub struct Filter {
    args: Vec<String>,
}

impl Filter {
    #[inline]
    pub fn any() -> db::Filter {
        db::Filter::any()
    }

    pub fn parse_optional(&self) -> Result<db::Filter> {
        db::Filter::parse_optional(&self.args)
    }

    pub fn parse(&self) -> Result<db::Filter> {
        db::Filter::parse(&self.args)
    }
}
