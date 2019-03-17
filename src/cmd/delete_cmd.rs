use crate::errors::*;

use crate::filters::{Target, Filter};
use crate::shell::Readline;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::models::*;
use crate::term;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(subcommand)]
    subcommand: Target,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    let rows = match args.subcommand {
        Target::Domains(filter) => delete::<Domain>(rl, &filter),
        Target::Subdomains(filter) => delete::<Subdomain>(rl, &filter),
        Target::IpAddrs(filter) => delete::<IpAddr>(rl, &filter),
        Target::Urls(filter) => delete::<Url>(rl, &filter),
        Target::Emails(filter) => delete::<Email>(rl, &filter),
        Target::PhoneNumbers(filter) => delete::<PhoneNumber>(rl, &filter),
        Target::Devices(filter) => delete::<Device>(rl, &filter),
        Target::Networks(filter) => delete::<Network>(rl, &filter),
        Target::Accounts(filter) => delete::<Account>(rl, &filter),
        Target::Breaches(filter) => delete::<Breach>(rl, &filter),
        Target::Images(filter) => delete::<Image>(rl, &filter),
    }?;
    term::info(&format!("Deleted {} rows", rows));
    Ok(())
}

#[inline]
fn delete<T: Model + Detailed>(rl: &mut Readline, filter: &Filter) -> Result<usize> {
    T::delete(rl.db(), &filter.parse()?)
}
