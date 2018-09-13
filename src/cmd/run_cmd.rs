use errors::*;

use shell::Readline;
use structopt::StructOpt;
use worker;


#[derive(Debug, StructOpt)]
pub struct Args {
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let _args = Args::from_iter_safe(args)?;

    if let Some(module) = rl.module() {
        worker::spawn(module);
    } else {
        eprintln!("Error: no module selected");
    }

    Ok(())
}
