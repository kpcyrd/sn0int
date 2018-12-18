use crate::errors::*;

use crate::args;
use crate::args::Install;
use crate::api::Client;
use crate::config::Config;
use colored::Colorize;
use crate::engine::Module;
use crate::registry;
use crate::shell::Readline;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::term;
use crate::worker;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    #[structopt(author="", name="list")]
    /// List installed modules
    List(List),
    #[structopt(author="", name="install")]
    /// Install module from registry
    Install(args::Install),
    #[structopt(author="", name="search")]
    /// Search modules in registry
    Search(args::Search),
    #[structopt(author="", name="reload")]
    /// Reload modules
    Reload(Reload),
    #[structopt(author="", name="update")]
    /// Update modules
    Update(Update),
}

#[derive(Debug, StructOpt)]
pub struct List {
}

#[derive(Debug, StructOpt)]
pub struct Reload {
}

#[derive(Debug, StructOpt)]
pub struct Update {
}

fn update(client: &Client, config: &Config, module: &Module) -> Result<()> {
    let installed = module.version();
    let infos = client.query_module(&module.id())?;
    debug!("Latest version: {:?}", infos);

    let latest = infos.latest.ok_or_else(|| format_err!("Module doesn't have any released versions"))?;

    if installed != latest {
        term::info(&format!("Updating {}: {:?} -> {:?}", module.canonical(), installed, latest));
        registry::run_install(&Install {
            module: module.id(),
            version: None,
        }, &config)?;
    }

    Ok(())
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    let config = rl.config().clone();

    match args.subcommand {
        SubCommand::List(_) => {
            for module in rl.engine().list() {
                println!("{} ({})", module.canonical().green(),
                                    module.version().yellow());
                println!("\t{}", module.description());
            }
        },
        SubCommand::Install(install) => {
            registry::run_install(&install, &config)?;
            // trigger reload
            run(rl, &[String::from("mod"), String::from("reload")])?;
        },
        SubCommand::Search(search) => registry::run_search(&search, &config)?,
        SubCommand::Reload(_) => {
            let current = rl.take_module()
                            .map(|m| m.canonical());

            rl.engine_mut().reload_modules()?;
            rl.reload_module_cache();

            if let Some(module) = current {
                if let Ok(module) = rl.engine().get(&module).map(|x| x.to_owned()) {
                    rl.set_module(module);
                }
            }
        },
        SubCommand::Update(_) => {
            let client = Client::new(&config)?;

            for module in rl.engine().list() {
                let name = module.canonical();
                let label = format!("Searching for updates {}", name);

                let result = worker::spawn_fn(&label, || {
                    update(&client, &config, &module)
                }, true);

                if let Err(err) = result {
                    term::error(&format!("Failed to update {}: {:?}", name, err));
                }
            }

            // trigger reload
            run(rl, &[String::from("mod"), String::from("reload")])?;
        },
    }

    Ok(())
}
