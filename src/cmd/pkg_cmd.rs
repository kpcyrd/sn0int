use crate::errors::*;
use crate::args::Install;
use crate::api::Client;
use crate::args;
use crate::config::Config;
use crate::cmd::{Cmd, LiteCmd};
use crate::engine::{Library, Module};
use crate::registry::{self, InstallTask, UpdateTask, Updater};
use crate::shell::Shell;
use crate::update::AutoUpdater;
use crate::worker;
use colored::{Color, Colorize};
use sn0int_common::ModuleID;
use sn0int_common::metadata::Stealth;
use std::collections::HashSet;
use std::fmt::Write;
use std::sync::Arc;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Parser)]
pub struct ArgsInteractive {
    #[command(subcommand)]
    pub subcommand: SubCommandInteractive,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    /// List installed modules
    #[command(name="list")]
    List(List),
    /// Install module from registry
    #[command(name="install")]
    Install(args::Install),
    /// Search modules in registry
    #[command(name="search")]
    Search(args::Search),
    /// Update modules
    #[command(name="update")]
    Update(Update),
    /// Uninstall a module
    #[command(name="uninstall")]
    Uninstall(Uninstall),
    /// Install all featured modules
    #[command(name="quickstart")]
    Quickstart,
}

#[derive(Debug, Parser)]
pub enum SubCommandInteractive {
    #[command(flatten)]
    Base(SubCommand),
    /// Reload modules
    #[command(name="reload")]
    Reload(Reload),
}

#[derive(Debug, Parser)]
pub struct List {
    /// Only show modules with a specific input source
    #[arg(long="source")]
    pub source: Option<String>,
    /// List outdated modules
    #[arg(long="outdated")]
    pub outdated_only: bool,
    /// Only show modules with equal or better stealth level
    #[arg(long="stealth", value_enum)]
    pub stealth: Option<Stealth>,
    /// Filter by pattern
    #[arg(default_value="*")]
    pub pattern: String,
}

#[derive(Debug, Parser)]
pub struct Reload {
}

#[derive(Debug, Parser)]
pub struct Update {
}

#[derive(Debug, Parser)]
pub struct Uninstall {
    module: ModuleID,
}

#[derive(PartialEq)]
enum ModuleReload {
    Yes,
    No,
}

#[inline]
fn write_tag(out: &mut String, color: Color, txt: &str) -> Result<()> {
    write!(out, " [{}]", txt.color(color))?;
    Ok(())
}

fn print_module(module: &Module, is_outdated: bool) -> Result<()> {
    let mut out = String::new();
    write!(&mut out, "{}/{} {}", module.author().purple(),
                                 module.name(),
                                 module.version().blue())?;

    match module.stealth() {
        Stealth::Loud => write_tag(&mut out, Color::Yellow, "LOUD")?,
        Stealth::Normal => (),
        Stealth::Passive => write_tag(&mut out, Color::Green, "passive")?,
        Stealth::Offline => write_tag(&mut out, Color::Green, "offline")?,
    };

    if is_outdated {
        write_tag(&mut out, Color::Red, "outdated")?;
    }

    println!("{}", out.bold());
    println!("    {}", module.description());
    Ok(())
}

fn run_subcommand(subcommand: SubCommand, library: &Library, config: &Config) -> Result<ModuleReload> {
    match subcommand {
        SubCommand::List(list) => {
            let autoupdate = AutoUpdater::load()?;

            let filter = glob::Pattern::new(&list.pattern)?;

            for module in library.list() {
                if let Some(source) = &list.source {
                    if !module.source_equals(source) {
                        continue;
                    }
                }

                let canonical = module.canonical();
                if !filter.matches(&canonical) {
                    continue;
                }

                if let Some(stealth) = &list.stealth {
                    if !module.stealth().equal_or_better(stealth) {
                        continue;
                    }
                }

                let is_outdated = autoupdate.is_outdated(&canonical);
                if list.outdated_only && !is_outdated {
                    continue;
                }

                print_module(module, is_outdated)?;
            }
            Ok(ModuleReload::No)
        },
        SubCommand::Install(install) => {
            registry::run_install(install, config)?;
            // trigger reload
            Ok(ModuleReload::Yes)
        },
        SubCommand::Search(search) => {
            registry::run_search(library, &search, config)?;
            Ok(ModuleReload::No)
        },
        SubCommand::Update(_) => {
            let mut autoupdate = AutoUpdater::load()?;
            let updater = Arc::new(Updater::new(config)?);

            let modules = library.list()
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
            let updater = Updater::new(config)?;
            updater.uninstall(&uninstall.module)?;
            // trigger reload
            Ok(ModuleReload::Yes)
        },
        SubCommand::Quickstart => {
            let client = Client::new(config)?;
            let updater = Arc::new(Updater::new(config)?);
            let mut autoupdate = AutoUpdater::load()?;

            let installed = library.list()
                .into_iter()
                .map(|module| module.id())
                .collect::<HashSet<_>>();

            let modules = client.quickstart()?
                .into_iter()
                .filter_map(|module| {
                    let id = ModuleID {
                        author: module.author,
                        name: module.name,
                    };

                    if !installed.contains(&id) {
                        info!("Queueing for install: {}", id);
                        Some(InstallTask::new(Install {
                            module: id,
                            version: None,
                            force: false,
                        }, updater.clone()))
                    } else {
                        info!("Skipping already installed module: {}", id);
                        None
                    }
                })
                .collect::<Vec<_>>();

            worker::spawn_multi(modules, |name| {
                autoupdate.updated(&name);
            }, 3)?;

            autoupdate.save()?;

            // trigger reload
            Ok(ModuleReload::Yes)
        },
    }
}

impl LiteCmd for Args {
    fn run(self, config: &Config) -> Result<()> {
        let library = Library::new(false, config)?;
        run_subcommand(self.subcommand, &library, config)?;
        Ok(())
    }
}

impl Cmd for ArgsInteractive {
    fn run(self, rl: &mut Shell) -> Result<()> {
        let action = match self.subcommand {
            SubCommandInteractive::Base(subcommand) => run_subcommand(subcommand, rl.library(), rl.config())?,
            SubCommandInteractive::Reload(_) => ModuleReload::Yes,
        };
        if action == ModuleReload::Yes {
            rl.reload_modules()?;
        }
        Ok(())
    }
}
