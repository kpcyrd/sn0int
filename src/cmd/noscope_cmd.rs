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
        Target::Domains(filter) => noscope::<Domain>(rl, &filter),
        Target::Subdomains(filter) => noscope::<Subdomain>(rl, &filter),
        Target::IpAddrs(filter) => noscope::<IpAddr>(rl, &filter),
        Target::Urls(filter) => noscope::<Url>(rl, &filter),
        Target::Emails(filter) => noscope::<Email>(rl, &filter),
        Target::PhoneNumbers(filter) => noscope::<PhoneNumber>(rl, &filter),
        Target::Devices(filter) => noscope::<Device>(rl, &filter),
        Target::Networks(filter) => noscope::<Network>(rl, &filter),
        Target::Accounts(filter) => noscope::<Account>(rl, &filter),
        Target::Breaches(filter) => noscope::<Breach>(rl, &filter),
        Target::Images(filter) => noscope::<Image>(rl, &filter),
    }?;
    term::info(&format!("Updated {} rows", rows));
    Ok(())
}

#[inline]
fn noscope<T: Model + Detailed>(rl: &mut Readline, filter: &Filter) -> Result<usize> {
    T::noscope(rl.db(), &filter.parse()?)
}
