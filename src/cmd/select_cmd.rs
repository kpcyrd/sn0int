use crate::errors::*;

use crate::cmd::Cmd;
use crate::db::ttl;
use crate::filters::{Filter, Target};
use crate::models::*;
use crate::shell::Shell;
use serde::Serialize;
use serde_json;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    #[structopt(subcommand)]
    subcommand: Target,
    /// Print json output
    #[structopt(long = "json", group = "output")]
    json: bool,
    /// Print paths to blobs
    #[structopt(long = "paths", group = "output")]
    paths: bool,
    /// Count rows returned
    #[structopt(short = "c", long = "count", group = "output")]
    count: bool,
}

#[derive(PartialEq)]
enum Output {
    Normal,
    Json,
    Paths,
    Count,
}

struct Printer<'a, 'b> {
    rl: &'a mut Shell<'b>,
    output: Output,
}

impl<'a, 'b> Printer<'a, 'b> {
    pub fn new(rl: &'a mut Shell<'b>, args: &Args) -> Printer<'a, 'b> {
        let output = if args.json {
            Output::Json
        } else if args.paths {
            Output::Paths
        } else if args.count {
            Output::Count
        } else {
            Output::Normal
        };

        Printer { rl, output }
    }

    pub fn select<T: Model + Detailed + Serialize>(&self, filter: &Filter) -> Result<()> {
        let query = self.rl.db().filter::<T>(&filter.parse_optional()?)?;

        if self.output == Output::Count {
            println!("{}", query.len());
        } else {
            for obj in query {
                match self.output {
                    Output::Normal => println!("{}", obj.detailed(self.rl.db())?),
                    Output::Json => {
                        let v = serde_json::to_string(&obj)?;
                        println!("{}", v);
                    }
                    Output::Paths => {
                        let blob = obj.blob().ok_or_else(|| {
                            format_err!("This model isn't linked to blob storage")
                        })?;

                        let path = self.rl.blobs().join(blob)?;

                        let path = path
                            .to_str()
                            .ok_or_else(|| format_err!("Path is invalid utf-8"))?;

                        println!("{}", path);
                    }
                    Output::Count => unreachable!(),
                }
            }
        }

        Ok(())
    }
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        ttl::reap_expired(rl)?;
        let printer = Printer::new(rl, &self);

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
            Target::Ports(filter) => printer.select::<Port>(&filter),
            Target::Netblocks(filter) => printer.select::<Netblock>(&filter),
            Target::CryptoAddrs(filter) => printer.select::<CryptoAddr>(&filter),
        }
    }
}
