use crate::errors::*;

use crate::shell::Shell;
use structopt::StructOpt;
use structopt::clap::AppSettings;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    module: String,
}

pub fn run(rl: &mut Shell, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;

    let module = rl.library().get(&args.module)?.clone();
    rl.set_module(module);

    Ok(())
}
