use crate::errors::*;

use crate::cmd::autonoscope_cmd;
use crate::shell::Shell;
use structopt::StructOpt;

pub type Args = autonoscope_cmd::Args;

pub fn run(rl: &mut Shell, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    autonoscope_cmd::run_with_scope_param(rl, args, true)
}
