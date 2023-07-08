use crate::errors::*;
use crate::cmd::Cmd;
use crate::cmd::pkg_cmd::{ArgsInteractive as PkgArgs, SubCommand, SubCommandInteractive};
use crate::shell::Shell;
use crate::term;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
}

pub fn run(rl: &mut Shell, args: &[String]) -> Result<()> {
    let _args = Args::try_parse_from(args)?;

    term::warn("The \x1b[1mquickstart\x1b[0m command is deprecated, use \x1b[1mpkg quickstart\x1b[0m");

    let args = PkgArgs {
        subcommand: SubCommandInteractive::Base(SubCommand::Quickstart),
    };
    args.run(rl)
}
