use crate::errors::*;

use crate::cmd::Cmd;
use crate::cmd::autonoscope_cmd;
use crate::shell::Shell;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    args: autonoscope_cmd::Args,
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        autonoscope_cmd::run_with_scope_param(rl, self.args, true)
    }
}
