use errors::*;

use db::{Database, Filter};
use sn0int_common::metadata::Source;
use serde::Serialize;
use serde_json;
use shell::Readline;
use structopt::StructOpt;
use models::*;
use term;
use worker;


#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(short="j", long="threads", default_value="1")]
    threads: usize,
    #[structopt(short="v", long="verbose", parse(from_occurrences))]
    verbose: u64,
}

fn prepare_arg<T: Serialize + Model>(x: T) -> Result<(serde_json::Value, Option<String>)> {
    let pretty = x.to_string();
    let arg = serde_json::to_value(x)?;
    Ok((arg, Some(pretty)))
}

fn prepare_args<T: Scopable + Serialize + Model>(db: &Database, filter: &Filter) -> Result<Vec<(serde_json::Value, Option<String>)>> {
    db.filter::<T>(filter)?
        .into_iter()
        .map(prepare_arg)
        .collect()
}

pub fn execute(rl: &mut Readline, threads: usize, verbose: u64) -> Result<()> {
    let module = rl.module()
        .map(|m| m.to_owned())
        .ok_or_else(|| format_err!("No module selected"))?;

    let filter = rl.scoped_targets();

    let args = match module.source() {
        Some(Source::Domains) => prepare_args::<Domain>(rl.db(), &filter),
        Some(Source::Subdomains) => prepare_args::<Subdomain>(rl.db(), &filter),
        Some(Source::IpAddrs) => prepare_args::<IpAddr>(rl.db(), &filter),
        Some(Source::Urls) => prepare_args::<Url>(rl.db(), &filter),
        Some(Source::Emails) => prepare_args::<Email>(rl.db(), &filter),
        None => Ok(vec![(serde_json::Value::Null, None)]),
    }?;

    rl.signal_register().catch_ctrl();
    worker::spawn(rl, &module, args, threads, verbose);
    rl.signal_register().reset_ctrlc();

    term::info(&format!("Finished {}", module.canonical()));

    Ok(())
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    execute(rl, args.threads, args.verbose)
}
