use errors::*;

use engine::metadata::Argument;
use serde;
use serde_json;
use shell::Readline;
use std::fmt;
use std::result;
use structopt::StructOpt;
use term;
use worker;


#[derive(Debug, StructOpt)]
pub struct Args {
}

fn prepare_arg<T: serde::Serialize + fmt::Display>(x: T) -> Result<(serde_json::Value, String)> {
    let pretty = x.to_string();
    let arg = serde_json::to_value(x)?;
    Ok((arg, pretty))
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let _args = Args::from_iter_safe(args)?;

    let module = rl.module()
        .map(|m| m.to_owned())
        .ok_or_else(|| format_err!("No module selected"))?;

    let args: result::Result<Vec<_>, _> = match module.argument() {
        Argument::Domains => rl.db().list_domains()?
                                .into_iter()
                                .map(prepare_arg)
                                .collect(),
        Argument::Subdomains => rl.db().list_subdomains()?
                                .into_iter()
                                .map(prepare_arg)
                                .collect(),
    };

    for (arg, pretty_arg) in args? {
        worker::spawn(rl, module.clone(), arg, &pretty_arg);
    }
    term::info(&format!("Finished {}", module.canonical()));

    Ok(())
}
