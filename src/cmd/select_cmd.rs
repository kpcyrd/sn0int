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
    /// Print json output
    #[structopt(long="json", group="output")]
    json: bool,
    /// Print paths to blobs
    #[structopt(long="paths", group="output")]
    paths: bool,
}

enum Output {
    Normal,
    Json,
    Paths,
}

struct Printer<'a, 'b> {
    rl: &'a mut Readline<'b>,
    output: Output,
}

impl<'a, 'b> Printer<'a, 'b> {
    pub fn new(rl: &'a mut Readline<'b>, args: &Args) -> Printer<'a, 'b> {
        let output = if args.json {
            Output::Json
        } else if args.paths {
            Output::Paths
        } else {
            Output::Normal
        };

        Printer {
            rl,
            output,
        }
    }

    pub fn select<T: Model + Detailed + Serialize>(&self, filter: &Filter) -> Result<()> {
        for obj in self.rl.db().filter::<T>(&filter.parse_optional()?)? {
            match self.output {
                Output::Normal => println!("{}", obj.detailed(self.rl.db())?),
                Output::Json => {
                    let v = serde_json::to_string(&obj)?;
                    println!("{}", v);
                },
                Output::Paths => {
                    let blob = obj.blob()
                        .ok_or_else(|| format_err!("This model isn't linked to blob storage"))?;

                    let path = self.rl.blobs()
                        .join(blob)?;

                    let path = path.to_str()
                        .ok_or_else(|| format_err!("Path is invalid utf-8"))?;

                    println!("{}", path);
                },
            }
        }

        Ok(())
    }
}

impl Cmd for Args {
    fn run(self, rl: &mut Readline) -> Result<()> {
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
        }
    }
}

#[inline]
pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    ttl::reap_expired(rl.db())?;
    Args::run_str(rl, args)
}
