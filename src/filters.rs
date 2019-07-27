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
}

#[derive(Debug, StructOpt)]
pub struct Filter {
    args: Vec<String>,
}

impl Filter {
    pub fn parse_optional(&self) -> Result<db::Filter> {
        db::Filter::parse_optional(&self.args)
    }

    pub fn parse(&self) -> Result<db::Filter> {
        db::Filter::parse(&self.args)
    }
}
