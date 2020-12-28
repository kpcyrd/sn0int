use crate::errors::*;

use crate::blobs::BlobStorage;
use crate::cmd::{Cmd, LiteCmd};
use crate::config::Config;
use crate::db::Database;
use crate::shell::Shell;
use crate::term;
use crate::utils;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::workspaces::{self, Workspace};


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Delete a workspaceb
    #[structopt(long = "delete", group = "action")]
    delete: bool,
    /// Show disk usage of workspace
    #[structopt(long = "usage", group = "action")]
    usage: bool,
    /// Skip confirmation
    #[structopt(short = "f", long = "force")]
    force: bool,
    workspace: Option<Workspace>,
}

fn delete(workspace: Workspace, force: bool) -> Result<()> {
    if !force && !utils::no_else_yes(&format!("Do you really want to delete {:?}", workspace.as_str()))? {
        return Ok(());
    }

    term::info(&format!("Deleting workspace: {:?}", workspace.as_str()));
    workspace.delete()?;

    Ok(())
}

fn usage(workspace: Option<Workspace>) -> Result<()> {
    if let Some(ws) = workspace {
        println!("{}", ws.usage_human()?);
    } else {
        for ws in workspaces::list()? {
            println!("{:50}: {}", ws.as_str(), ws.usage_human()?);
        }
    }

    Ok(())
}

fn change(rl: &mut Shell, workspace: Workspace) -> Result<()> {
    workspace.migrate()?;

    let blobs = BlobStorage::workspace(&workspace)?;
    let db = Database::establish(workspace)?;
    rl.set_blobstorage(blobs);
    rl.set_db(db);
    Ok(())
}

fn list() -> Result<()> {
    for ws in workspaces::list()? {
        println!("{}", ws.as_str());
    }
    Ok(())
}

fn run(args: Args, rl: Option<&mut Shell>) -> Result<()> {
    if args.delete {
        if let Some(workspace) = args.workspace {
            if let Some(rl) = rl {
                if *rl.db().workspace() == workspace {
                    bail!("Can't delete current workspace")
                }
            }

            delete(workspace, args.force)
        } else {
            bail!("--delete requires workspace")
        }
    } else if args.usage {
        usage(args.workspace)
    } else if let Some(workspace) = args.workspace {
        if let Some(rl) = rl {
            change(rl, workspace)
        } else {
            Ok(())
        }
    } else {
        list()
    }
}

impl Cmd for Args {
    #[inline]
    fn run(self, rl: &mut Shell) -> Result<()> {
        run(self, Some(rl))
    }
}

impl LiteCmd for Args {
    #[inline]
    fn run(self, _config: &Config) -> Result<()> {
        run(self, None)
    }
}
