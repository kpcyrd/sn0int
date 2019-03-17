use crate::errors::*;

use crate::db;
use crate::shell::Readline;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::models::*;
use crate::term;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub enum Args {
    #[structopt(name="domains")]
    Domains(Filter),
    #[structopt(name="subdomains")]
    Subdomains(Filter),
    #[structopt(name="ipaddrs")]
    IpAddrs(Filter),
    #[structopt(name="urls")]
    Urls(Filter),
    #[structopt(name="emails")]
    Emails(Filter),
    #[structopt(name="phonenumbers")]
    PhoneNumbers(Filter),
    #[structopt(name="devices")]
    Devices(Filter),
    #[structopt(name="networks")]
    Networks(Filter),
    #[structopt(name="accounts")]
    Accounts(Filter),
    #[structopt(name="breaches")]
    Breaches(Filter),
    #[structopt(name="images")]
    Images(Filter),
}

#[derive(Debug, StructOpt)]
pub struct Filter {
    args: Vec<String>,
}

impl Filter {
    pub fn parse(&self) -> Result<db::Filter> {
        db::Filter::parse(&self.args)
    }
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    let rows = match args {
        Args::Domains(filter) => delete::<Domain>(rl, &filter),
        Args::Subdomains(filter) => delete::<Subdomain>(rl, &filter),
        Args::IpAddrs(filter) => delete::<IpAddr>(rl, &filter),
        Args::Urls(filter) => delete::<Url>(rl, &filter),
        Args::Emails(filter) => delete::<Email>(rl, &filter),
        Args::PhoneNumbers(filter) => delete::<PhoneNumber>(rl, &filter),
        Args::Devices(filter) => delete::<Device>(rl, &filter),
        Args::Networks(filter) => delete::<Network>(rl, &filter),
        Args::Accounts(filter) => delete::<Account>(rl, &filter),
        Args::Breaches(filter) => delete::<Breach>(rl, &filter),
        Args::Images(filter) => delete::<Image>(rl, &filter),
    }?;
    term::info(&format!("Deleted {} rows", rows));
    Ok(())
}

#[inline]
fn delete<T: Model + Detailed>(rl: &mut Readline, filter: &Filter) -> Result<usize> {
    T::delete(rl.db(), &filter.parse()?)
}
