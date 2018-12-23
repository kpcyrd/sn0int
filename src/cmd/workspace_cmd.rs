use crate::errors::*;

use crate::db::Database;
use crate::shell::Readline;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::workspaces::{self, Workspace};


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    workspace: Option<Workspace>,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;

    if let Some(workspace) = args.workspace {
        let db = Database::establish(workspace)?;
        rl.set_db(db);
    } else {
        for x in workspaces::list()? {
            println!("{:?}", x);
        }
    }

    Ok(())
}
