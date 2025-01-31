use clap::Parser;
use crate::blobs::BlobStorage;
use crate::cmd::{Cmd, LiteCmd};
use crate::config::Config;
use crate::db::Database;
use crate::errors::*;
use crate::shell::Shell;
use crate::term;
use crate::utils;
use crate::workspaces::{self, Workspace};

#[derive(Debug, Parser)]
pub struct Args {
    /// Delete a workspace
    #[arg(long = "delete", group = "action")]
    delete: bool,
    /// Show disk usage of workspace
    #[arg(long = "usage", group = "action")]
    usage: bool,
    /// Skip confirmation
    #[arg(short = 'f', long = "force")]
    force: bool,
    workspaces: Vec<Workspace>,
}

fn delete(workspace: Workspace, force: bool) -> Result<()> {
    if !force && !utils::no_else_yes(&format!("Do you really want to delete {:?}", workspace.as_str()))? {
        return Ok(());
    }

    term::info(&format!("Deleting workspace: {:?}", workspace.as_str()));
    workspace.delete()?;

    Ok(())
}

fn usage(mut workspaces: Vec<Workspace>) -> Result<()> {
    if workspaces.is_empty() {
        workspaces = workspaces::list()?;
    }

    for ws in workspaces {
        println!("{:50}: {}", ws.as_str(), ws.usage_human()?);
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

fn run(mut args: Args, rl: Option<&mut Shell>) -> Result<()> {
    if args.delete {
        if args.workspaces.is_empty() {
            bail!("--delete requires workspace");
        }

        for workspace in args.workspaces {
            if let Some(rl) = &rl {
                if *rl.db().workspace() == workspace {
                    bail!("Can't delete current workspace")
                }
            }
            delete(workspace, args.force)?;
        }
    } else if args.usage {
        usage(args.workspaces)?;
    } else {
        match args.workspaces.len() {
            0 => list()?,
            1 => if let Some(rl) = rl {
                // we've already tested there's one item in the list
                change(rl, args.workspaces.pop().unwrap())?;
            },
            _ => bail!("Only one argument allowed when switching workspaces"),
        }
    }

    Ok(())
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
