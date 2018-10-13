use errors::*;

use args;
use colored::Colorize;
use registry;
use shell::Readline;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
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
}

#[derive(Debug, StructOpt)]
pub struct List {
}

#[derive(Debug, StructOpt)]
pub struct Reload {
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
    }

    Ok(())
}
