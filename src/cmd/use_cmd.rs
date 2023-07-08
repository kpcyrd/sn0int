use clap::Parser;
use crate::errors::*;
use crate::shell::Shell;

#[derive(Debug, Parser)]
pub struct Args {
    module: String,
}

pub fn run(rl: &mut Shell, args: &[String]) -> Result<()> {
    let args = Args::try_parse_from(args)?;

    let module = rl.library().get(&args.module)?.clone();
    rl.set_module(module);

    Ok(())
}
