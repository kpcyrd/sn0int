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
        .ok_or_else(|| format_err!("No module selected"))?;

    worker::spawn(module.to_owned());

    Ok(())
}
