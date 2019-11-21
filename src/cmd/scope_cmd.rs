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
        let rows = match self.subcommand {
            Target::Domains(filter) => scope::<Domain>(rl, &filter),
            Target::Subdomains(filter) => scope::<Subdomain>(rl, &filter),
            Target::IpAddrs(filter) => scope::<IpAddr>(rl, &filter),
            Target::Urls(filter) => scope::<Url>(rl, &filter),
            Target::Emails(filter) => scope::<Email>(rl, &filter),
            Target::PhoneNumbers(filter) => scope::<PhoneNumber>(rl, &filter),
            Target::Devices(filter) => scope::<Device>(rl, &filter),
            Target::Networks(filter) => scope::<Network>(rl, &filter),
            Target::Accounts(filter) => scope::<Account>(rl, &filter),
            Target::Breaches(filter) => scope::<Breach>(rl, &filter),
            Target::Images(filter) => scope::<Image>(rl, &filter),
            Target::Ports(filter) => scope::<Port>(rl, &filter),
            Target::Netblocks(filter) => scope::<Netblock>(rl, &filter),
            Target::CryptoAddrs(filter) => scope::<CryptoAddr>(rl, &filter),
        }?;
        term::info(&format!("Updated {} rows", rows));
        Ok(())
    }
}

pub fn run(rl: &mut Shell, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    args.run(rl)
}

#[inline]
fn scope<T: Model + Detailed>(rl: &mut Shell, filter: &Filter) -> Result<usize> {
    T::scope(rl.db(), &filter.parse()?)
}
