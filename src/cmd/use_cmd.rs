use crate::errors::*;

use crate::shell::Readline;
use structopt::StructOpt;
use structopt::clap::AppSettings;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    module: String,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;

    let module = rl.engine().get(&args.module)?.clone();
    rl.set_module(module);

    Ok(())
}
