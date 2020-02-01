use crate::errors::*;

use crate::args;
use crate::config::Config;
use crate::cmd::{Cmd, LiteCmd};
use crate::engine::Engine;
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

#[derive(PartialEq)]
enum ModuleReload {
    Yes,
    No,
}

fn run_subcommand(subcommand: SubCommand, engine: &Engine, config: &Config) -> Result<ModuleReload> {
    match subcommand {
        SubCommand::List(list) => {
            let autoupdate = AutoUpdater::load()?;

            for module in engine.list() {
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
            Ok(ModuleReload::No)
        },
        SubCommand::Install(install) => {
            registry::run_install(install, &config)?;
            // trigger reload
            Ok(ModuleReload::Yes)
        },
        SubCommand::Search(search) => {
            registry::run_search(engine, &search, &config)?;
            Ok(ModuleReload::No)
        },
        SubCommand::Update(_) => {
            let mut autoupdate = AutoUpdater::load()?;
            let updater = Arc::new(Updater::new(&config)?);

            let modules = engine.list()
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
            Ok(ModuleReload::Yes)
        },
        SubCommand::Uninstall(uninstall) => {
            let updater = Updater::new(&config)?;
            updater.uninstall(&uninstall.module)?;
            // trigger reload
            Ok(ModuleReload::Yes)
        },
    }
}

impl LiteCmd for Args {
    fn run(self, config: &Config) -> Result<()> {
        let engine = Engine::new(false, &config)?;
        run_subcommand(self.subcommand, &engine, config)?;
        Ok(())
    }
}

impl Cmd for ArgsInteractive {
    fn run(self, rl: &mut Shell) -> Result<()> {
        let action = match self.subcommand {
            SubCommandInteractive::Base(subcommand) => run_subcommand(subcommand, rl.engine(), rl.config())?,
            SubCommandInteractive::Reload(_) => ModuleReload::Yes,
        };
        if action == ModuleReload::Yes {
            rl.reload_modules()?;
        }
        Ok(())
    }
}
