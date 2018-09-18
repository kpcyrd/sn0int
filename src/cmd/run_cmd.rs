use errors::*;

use engine::metadata::Argument;
use serde_json;
use shell::Readline;
use std::result;
use structopt::StructOpt;
use worker;


#[derive(Debug, StructOpt)]
pub struct Args {
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let _args = Args::from_iter_safe(args)?;

    let module = rl.module()
        .map(|m| m.to_owned())
        .ok_or_else(|| format_err!("No module selected"))?;

    let args: result::Result<Vec<_>, _> = match module.argument() {
        Argument::Domains => rl.db().list_domains()?
                                .into_iter()
                                .map(|x| serde_json::to_value(x))
                                .collect(),
        Argument::Subdomains => rl.db().list_subdomains()?
                                .into_iter()
                                .map(|x| serde_json::to_value(x))
                                .collect(),
    };

    for arg in args? {
        worker::spawn(rl, module.clone(), arg);
    }

    Ok(())
}
