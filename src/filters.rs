use crate::errors::*;

use crate::db;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub enum Target {
    /// On domains
    #[structopt(name="domains")]
    Domains(Filter),
    /// On subdomains
    #[structopt(name="subdomains")]
    Subdomains(Filter),
    /// On ipaddrs
    #[structopt(name="ipaddrs")]
    IpAddrs(Filter),
    /// On urls
    #[structopt(name="urls")]
    Urls(Filter),
    /// On emails
    #[structopt(name="emails")]
    Emails(Filter),
    /// On phone numbers
    #[structopt(name="phonenumbers")]
    PhoneNumbers(Filter),
    /// On devices
    #[structopt(name="devices")]
    Devices(Filter),
    /// On networks
    #[structopt(name="networks")]
    Networks(Filter),
    /// On accounts
    #[structopt(name="accounts")]
    Accounts(Filter),
    /// On breaches
    #[structopt(name="breaches")]
    Breaches(Filter),
    /// On images
    #[structopt(name="images")]
    Images(Filter),
    /// On ports
    #[structopt(name="ports")]
    Ports(Filter),
    /// On ipnets
    #[structopt(name="netblocks")]
    Netblocks(Filter),
    /// On crypto currency addresses
    #[structopt(name="cryptoaddrs")]
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

#[derive(Debug, StructOpt)]
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
