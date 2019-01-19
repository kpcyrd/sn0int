use crate::errors::*;

use crate::cmd::Cmd;
use crate::db;
use crate::shell::Readline;
use serde::Serialize;
use serde_json;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::models::*;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(subcommand)]
    subcommand: Target,
    #[structopt(long="json")]
    /// Json output
    json: bool,
}

#[derive(Debug, StructOpt)]
pub enum Target {
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

pub struct Printer<'a> {
    rl: &'a mut Readline,
    json: bool,
}

impl<'a> Printer<'a> {
    pub fn new(rl: &'a mut Readline, json: bool) -> Printer<'a> {
        Printer {
            rl,
            json,
        }
    }

    pub fn select<T: Model + Detailed + Serialize>(&self, filter: &Filter) -> Result<()> {
        for obj in self.rl.db().filter::<T>(&filter.parse()?)? {
            if self.json {
                let v = serde_json::to_string(&obj)?;
                println!("{}", v);
            } else {
                println!("{}", obj.detailed(self.rl.db())?);
            }
        }

        Ok(())
    }
}

impl Cmd for Args {
    fn run(&self, rl: &mut Readline) -> Result<()> {
        let printer = Printer::new(rl, self.json);

        match &self.subcommand {
            Target::Domains(filter) => printer.select::<Domain>(&filter),
            Target::Subdomains(filter) => printer.select::<Subdomain>(&filter),
            Target::IpAddrs(filter) => printer.select::<IpAddr>(&filter),
            Target::Urls(filter) => printer.select::<Url>(&filter),
            Target::Emails(filter) => printer.select::<Email>(&filter),
            Target::PhoneNumbers(filter) => printer.select::<PhoneNumber>(&filter),
            Target::Devices(filter) => printer.select::<Device>(&filter),
            Target::Networks(filter) => printer.select::<Network>(&filter),
        }
    }
}

#[inline]
pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    Args::run_str(rl, args)
}
