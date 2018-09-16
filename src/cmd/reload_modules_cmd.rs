use errors::*;

use shell::Readline;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub struct Args {
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let _args = Args::from_iter_safe(args)?;

    rl.engine_mut().reload_modules()?;
    rl.reload_module_cache();

    Ok(())
}
