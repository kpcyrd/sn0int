use errors::*;

use shell::Readline;
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

    worker::spawn(rl, module);

    Ok(())
}
