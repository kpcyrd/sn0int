use crate::errors::*;

use crate::cmd::Cmd;
use crate::db::ttl;
use crate::filters::{Target, Filter};
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
    /// Json output
    #[structopt(long="json")]
    json: bool,
}

pub struct Printer<'a, 'b> {
    rl: &'a mut Readline<'b>,
    json: bool,
}

impl<'a, 'b> Printer<'a, 'b> {
    pub fn new(rl: &'a mut Readline<'b>, json: bool) -> Printer<'a, 'b> {
        Printer {
            rl,
            json,
        }
    }

    pub fn select<T: Model + Detailed + Serialize>(&self, filter: &Filter) -> Result<()> {
        for obj in self.rl.db().filter::<T>(&filter.parse_optional()?)? {
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
    fn run(self, rl: &mut Readline) -> Result<()> {
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
            Target::Accounts(filter) => printer.select::<Account>(&filter),
            Target::Breaches(filter) => printer.select::<Breach>(&filter),
            Target::Images(filter) => printer.select::<Image>(&filter),
        }
    }
}

#[inline]
pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    ttl::reap_expired(rl.db())?;
    Args::run_str(rl, args)
}
