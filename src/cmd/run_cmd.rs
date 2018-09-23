use errors::*;

use db::Database;
use engine::metadata::Argument;
use serde::Serialize;
use serde_json;
use shell::Readline;
use std::fmt;
use structopt::StructOpt;
use models::*;
use term;
use worker;


#[derive(Debug, StructOpt)]
pub struct Args {
}

fn prepare_arg<T: Serialize + fmt::Display>(x: T) -> Result<(serde_json::Value, String)> {
    let pretty = x.to_string();
    let arg = serde_json::to_value(x)?;
    Ok((arg, pretty))
}

fn prepare_args<T: Model + Serialize + fmt::Display>(db: &Database) -> Result<Vec<(serde_json::Value, String)>> {
    db.list::<T>()?
        .into_iter()
        .map(prepare_arg)
        .collect()
}

pub fn execute(rl: &mut Readline) -> Result<()> {
    let module = rl.module()
        .map(|m| m.to_owned())
        .ok_or_else(|| format_err!("No module selected"))?;

    let args = match module.argument() {
        Argument::Domains => prepare_args::<Domain>(rl.db()),
        Argument::Subdomains => prepare_args::<Subdomain>(rl.db()),
    };

    for (arg, pretty_arg) in args? {
        worker::spawn(rl, module.clone(), arg, &pretty_arg);
    }
    term::info(&format!("Finished {}", module.canonical()));

    Ok(())
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let _args = Args::from_iter_safe(args)?;
    execute(rl)
}
