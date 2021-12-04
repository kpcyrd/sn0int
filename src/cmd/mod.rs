use crate::errors::*;
use crate::shell::Shell;
use crate::config::Config;

pub trait Cmd: structopt::StructOpt + Sized {
    fn run(self, rl: &mut Shell) -> Result<()>;

    #[inline]
    fn run_str(rl: &mut Shell, args: &[String]) -> Result<()> {
        let args = Self::from_iter_safe(args)?;
        args.run(rl)
    }
}

pub trait LiteCmd: structopt::StructOpt + Sized {
    fn run(self, config: &Config) -> Result<()>;
}

pub mod activity_cmd;
pub mod add_cmd;
pub mod autonoscope_cmd;
pub mod autoscope_cmd;
pub mod cal_cmd;
pub mod delete_cmd;
pub mod export_cmd;
pub mod fsck_cmd;
pub mod help_cmd;
pub mod run_cmd;
pub mod use_cmd;
pub mod select_cmd;
pub mod keyring_cmd;
pub mod noscope_cmd;
pub mod notify_cmd;
pub mod pkg_cmd;
pub mod rescope_cmd;
pub mod set_cmd;
pub mod scope_cmd;
pub mod stats_cmd;
pub mod target_cmd;
pub mod quickstart_cmd;
pub mod workspace_cmd;
