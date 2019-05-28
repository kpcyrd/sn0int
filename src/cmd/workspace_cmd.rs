use crate::errors::*;

use crate::blobs::BlobStorage;
use crate::cmd::Cmd;
use crate::db::Database;
use crate::shell::Readline;
use crate::term;
use crate::utils;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::workspaces::{self, Workspace};


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
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
    if !force {
        if !utils::no_else_yes(&format!("Do you really want to delete {:?}", workspace.as_str()))? {
            return Ok(());
        }
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

fn change(rl: &mut Readline, workspace: Workspace) -> Result<()> {
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

impl Cmd for Args {
    fn run(self, rl: &mut Readline) -> Result<()> {
        if self.delete {
            if let Some(workspace) = self.workspace {
                if *rl.db().workspace() == workspace {
                    bail!("Can't delete current workspace")
                }

                delete(workspace, self.force)
            } else {
                bail!("--delete requires workspace")
            }
        } else if self.usage {
            usage(self.workspace)
        } else if let Some(workspace) = self.workspace {
            change(rl, workspace)
        } else {
            list()
        }
    }
}

#[inline]
pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    Args::run_str(rl, args)
}
