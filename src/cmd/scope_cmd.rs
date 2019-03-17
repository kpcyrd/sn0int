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
        Args::Domains(filter) => scope::<Domain>(rl, &filter),
        Args::Subdomains(filter) => scope::<Subdomain>(rl, &filter),
        Args::IpAddrs(filter) => scope::<IpAddr>(rl, &filter),
        Args::Urls(filter) => scope::<Url>(rl, &filter),
        Args::Emails(filter) => scope::<Email>(rl, &filter),
        Args::PhoneNumbers(filter) => scope::<PhoneNumber>(rl, &filter),
    }?;
    term::info(&format!("Updated {} rows", rows));
    Ok(())
}

#[inline]
fn scope<T: Model + Detailed>(rl: &mut Readline, filter: &Filter) -> Result<usize> {
    T::scope(rl.db(), &filter.parse()?)
}
