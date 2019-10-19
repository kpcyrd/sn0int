use crate::errors::*;

use crate::args::Install;
use crate::api::Client;
use crate::cmd::mod_cmd;
use crate::registry::{InstallTask, Updater};
use crate::shell::Shell;
use crate::update::AutoUpdater;
use crate::worker;
use std::sync::Arc;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use sn0int_common::ModuleID;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
}

pub fn run(rl: &mut Shell, args: &[String]) -> Result<()> {
    let _args = Args::from_iter_safe(args)?;
    let config = rl.config().clone();

    let client = Client::new(&config)?;
    let updater = Arc::new(Updater::new(&config)?);
    let mut autoupdate = AutoUpdater::load()?;

    let modules = client.quickstart()?
        .into_iter()
        .map(|module| {
            InstallTask::new(Install {
                module: ModuleID {
                    author: module.author,
                    name: module.name,
                },
                version: None,
                force: false,
            }, updater.clone())
        })
        .collect::<Vec<_>>();

    worker::spawn_multi(modules, |name| {
        autoupdate.updated(&name);
    }, 3)?;

    autoupdate.save()?;

    // trigger reload
    mod_cmd::run(rl, &[String::from("mod"), String::from("reload")])?;

    Ok(())
}
