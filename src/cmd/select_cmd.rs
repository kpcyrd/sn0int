use crate::errors::*;

use crate::db;
use crate::shell::Readline;
use structopt::StructOpt;
use crate::models::*;


#[derive(Debug, StructOpt)]
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
        db::Filter::parse_optional(&self.args)
    }
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    match args {
        Args::Domains(filter) => select::<Domain>(rl, &filter),
        Args::Subdomains(filter) => select::<Subdomain>(rl, &filter),
        Args::IpAddrs(filter) => select::<IpAddr>(rl, &filter),
        Args::Urls(filter) => select::<Url>(rl, &filter),
        Args::Emails(filter) => select::<Email>(rl, &filter),
        Args::PhoneNumbers(filter) => select::<PhoneNumber>(rl, &filter),
    }
}

fn select<T: Model + Detailed>(rl: &mut Readline, filter: &Filter) -> Result<()> {
    for obj in rl.db().filter::<T>(&filter.parse()?)? {
        println!("{}", obj.detailed(rl.db())?);
    }

    Ok(())
}
