use crate::errors::*;

use crate::cmd::Cmd;
use crate::db;
use crate::shell::Readline;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::models::*;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub enum Args {
    #[structopt(name="domains")]
    /// Select domains
    Domains(Filter),
    #[structopt(name="subdomains")]
    /// Select subdomains
    Subdomains(Filter),
    #[structopt(name="ipaddrs")]
    /// Select ipaddrs
    IpAddrs(Filter),
    #[structopt(name="urls")]
    /// Select urls
    Urls(Filter),
    #[structopt(name="emails")]
    /// Select emails
    Emails(Filter),
    #[structopt(name="phonenumbers")]
    /// Select phone numbers
    PhoneNumbers(Filter),
    #[structopt(name="devices")]
    /// Select devices
    Devices(Filter),
    #[structopt(name="networks")]
    /// Select networks
    Networks(Filter),
}

#[derive(Debug, StructOpt)]
pub struct Filter {
    args: Vec<String>,
}

impl Filter {
    pub fn parse(&self) -> Result<db::Filter> {
        db::Filter::parse_optional(&self.args)
    }
}

impl Cmd for Args {
    fn run(&self, rl: &mut Readline) -> Result<()> {
        match self {
            Args::Domains(filter) => select::<Domain>(rl, &filter),
            Args::Subdomains(filter) => select::<Subdomain>(rl, &filter),
            Args::IpAddrs(filter) => select::<IpAddr>(rl, &filter),
            Args::Urls(filter) => select::<Url>(rl, &filter),
            Args::Emails(filter) => select::<Email>(rl, &filter),
            Args::PhoneNumbers(filter) => select::<PhoneNumber>(rl, &filter),
            Args::Devices(filter) => select::<Device>(rl, &filter),
            Args::Networks(filter) => select::<Network>(rl, &filter),
        }
    }
}

#[inline]
pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    Args::run_str(rl, args)
}

fn select<T: Model + Detailed>(rl: &mut Readline, filter: &Filter) -> Result<()> {
    for obj in rl.db().filter::<T>(&filter.parse()?)? {
        println!("{}", obj.detailed(rl.db())?);
    }

    Ok(())
}
