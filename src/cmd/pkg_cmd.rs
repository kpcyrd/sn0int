use crate::errors::*;

use crate::args;
use crate::registry::{self, UpdateTask, Updater};
use crate::shell::Shell;
use crate::update::AutoUpdater;
use crate::worker;
use colored::Colorize;
use sn0int_common::ModuleID;
use std::fmt::Write;
use std::sync::Arc;
use structopt::StructOpt;
use structopt::clap::AppSettings;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct ArgsInteractive {
    #[structopt(subcommand)]
    pub subcommand: SubCommandInteractive,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    /// List installed modules
    #[structopt(name="list")]
    List(List),
    /// Install module from registry
    #[structopt(name="install")]
    Install(args::Install),
    /// Search modules in registry
    #[structopt(name="search")]
    Search(args::Search),
    /// Update modules
    #[structopt(name="update")]
    Update(Update),
    /// Uninstall a module
    #[structopt(name="uninstall")]
    Uninstall(Uninstall),
}

#[derive(Debug, StructOpt)]
pub enum SubCommandInteractive {
    #[structopt(flatten)]
    Base(SubCommand),
    /// Reload modules
    #[structopt(name="reload")]
    Reload(Reload),
}

#[derive(Debug, StructOpt)]
pub struct List {
    /// Only show modules with a specific input source
    #[structopt(long="source")]
    pub source: Option<String>,
    /// List outdated modules
    #[structopt(long="outdated")]
    pub outdated: bool,
}

#[derive(Debug, StructOpt)]
pub struct Reload {
}

#[derive(Debug, StructOpt)]
pub struct Update {
}

#[derive(Debug, StructOpt)]
pub struct Uninstall {
    module: ModuleID,
}

fn run_subcommand(rl: &mut Shell, subcommand: SubCommand) -> Result<()> {
    let config = rl.config().clone();

    match subcommand {
        SubCommand::List(list) => {
            let autoupdate = AutoUpdater::load()?;

            for module in rl.engine().list() {
                if let Some(source) = &list.source {
                    if !module.source_equals(&source) {
                        continue;
                    }
                }

                let canonical = module.canonical();

                let mut out = String::new();
                write!(&mut out, "{} ({})", canonical.green(),
                                            module.version().yellow())?;
                if autoupdate.is_outdated(&canonical) {
                    write!(&mut out, " {}", "[outdated]".red())?;
                } else if list.outdated {
                    continue;
                }
                println!("{}", out);
                println!("\t{}", module.description());
            }
        },
        SubCommand::Install(install) => {
            registry::run_install(install, &config)?;
            // trigger reload
            rl.reload_modules()?;
        },
        SubCommand::Search(search) => registry::run_search(rl.engine(), &search, &config)?,
        SubCommand::Update(_) => {
            let mut autoupdate = AutoUpdater::load()?;
            let updater = Arc::new(Updater::new(&config)?);

            let modules = rl.engine().list()
                .into_iter()
                .filter_map(|module| {
                    let canonical = module.canonical();

                    if module.is_private() {
                        debug!("{} is a private module, skipping", canonical);
                        return None;
                    }

                    Some(UpdateTask::new(module.clone(), updater.clone()))
                })
                .collect::<Vec<_>>();

            worker::spawn_multi(modules, |name| {
                autoupdate.updated(&name);
            }, 3)?;

            autoupdate.save()?;

            // trigger reload
            rl.reload_modules()?;
        },
        SubCommand::Uninstall(uninstall) => {
            let updater = Updater::new(&config)?;
            updater.uninstall(&uninstall.module)?;
            // trigger reload
            rl.reload_modules()?;
        },
    }

    Ok(())
}

pub fn run(rl: &mut Shell, args: &[String]) -> Result<()> {
    let args = ArgsInteractive::from_iter_safe(args)?;

    match args.subcommand {
        SubCommandInteractive::Base(subcommand) => run_subcommand(rl, subcommand),
        SubCommandInteractive::Reload(_) => rl.reload_modules(),
    }
}
