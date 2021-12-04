use crate::errors::*;

use crate::autonoscope;
use crate::cmd::Cmd;
use crate::fmt::colors::*;
use crate::shell::Shell;
use std::fmt::Write;
use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    #[structopt(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    #[structopt(name="add")]
    Add(Add),
    #[structopt(name="delete")]
    Delete(Delete),
    #[structopt(name="list")]
    List,
}

#[derive(Debug, StructOpt)]
pub struct Add {
    object: autonoscope::RuleType,
    value: String,
}

#[derive(Debug, StructOpt)]
pub struct Delete {
    object: autonoscope::RuleType,
    value: String,
}

fn display_rule<T: Color>(object: &str, rule: &str) -> Result<()> {
    let mut out = String::new();
    T::display(&mut out, object)?;
    write!(&mut out, " {:?}", rule)?;
    println!("{}", out);
    Ok(())
}

pub fn run_with_scope_param(rl: &mut Shell, args: Args, scoped: bool) -> Result<()> {
    match args.subcommand {
        Subcommand::Add(add) => {
            rl.db_mut().autonoscope_add_rule(&add.object, &add.value, scoped)
        },
        Subcommand::Delete(delete) => {
            rl.db_mut().autonoscope_delete_rule(&delete.object, &delete.value)
        },
        Subcommand::List => {
            for (object, rule, scoped) in rl.db().autonoscope_rules() {
                if scoped {
                    display_rule::<Green>(&format!("  scope {}", object), &rule)?;
                } else {
                    display_rule::<Red>(&format!("noscope {}", object), &rule)?;
                }
            }
            Ok(())
        },
    }
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        run_with_scope_param(rl, self, false)
    }
}
