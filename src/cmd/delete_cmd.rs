use crate::errors::*;

use crate::cmd::Cmd;
use crate::filters::{Target, Filter};
use crate::shell::Shell;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::models::*;
use crate::term;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    #[structopt(subcommand)]
    subcommand: Target,
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        let rows = match &self.subcommand {
            Target::Domains(filter) => delete::<Domain>(rl, filter),
            Target::Subdomains(filter) => delete::<Subdomain>(rl, filter),
            Target::IpAddrs(filter) => delete::<IpAddr>(rl, filter),
            Target::Urls(filter) => delete::<Url>(rl, filter),
            Target::Emails(filter) => delete::<Email>(rl, filter),
            Target::PhoneNumbers(filter) => delete::<PhoneNumber>(rl, filter),
            Target::Devices(filter) => delete::<Device>(rl, filter),
            Target::Networks(filter) => delete::<Network>(rl, filter),
            Target::Accounts(filter) => delete::<Account>(rl, filter),
            Target::Breaches(filter) => delete::<Breach>(rl, filter),
            Target::Images(filter) => delete::<Image>(rl, filter),
            Target::Ports(filter) => delete::<Port>(rl, filter),
            Target::Netblocks(filter) => delete::<Netblock>(rl, filter),
            Target::CryptoAddrs(filter) => delete::<CryptoAddr>(rl, filter),
        }?;
        term::info(&format!("Deleted {} rows", rows));
        Ok(())
    }
}

#[inline]
pub fn run(rl: &mut Shell, args: &[String]) -> Result<()> {
    Args::run_str(rl, args)
}

#[inline]
fn delete<T: Model + Detailed>(rl: &mut Shell, filter: &Filter) -> Result<usize> {
    T::delete(rl.db(), &filter.parse()?)
}
