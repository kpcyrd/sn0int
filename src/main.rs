#![warn(unused_extern_crates)]
extern crate sn0int;
extern crate env_logger;
#[macro_use] extern crate failure;
extern crate structopt;

use sn0int::args::{self, Args, SubCommand};
use sn0int::auth;
use sn0int::cmd;
use sn0int::complete;
use sn0int::config::Config;
use sn0int::errors::*;
use sn0int::engine::{self, Module};
use sn0int::registry;
use sn0int::sandbox;
use sn0int::shell;
use structopt::StructOpt;
use std::path::Path;


fn run_run(gargs: &Args, args: &args::Run, config: Config) -> Result<()> {
    let mut rl = shell::init(gargs, config)?;

    if let Some(module) = &args.module {
        let module = rl.engine().get(&module)?.clone();
        rl.set_module(module);
    } else if let Some(file) = &args.file {
        let path = Path::new(file);

        let filename = path.file_stem()
            .ok_or(format_err!("Failed to decode filename"))?
            .to_str()
            .ok_or(format_err!("Failed to decode filename"))?;

        let module = Module::load(&path.to_path_buf(), "anonymous", &filename)
            .context(format!("Failed to parse {:?}", file))?;
        rl.set_module(module);
    } else {
        bail!("At least one module or file need to be provided");
    }

    cmd::run_cmd::execute(&mut rl)
}

fn run_sandbox() -> Result<()> {
    sandbox::init()
        .context("Failed to init sandbox")?;
    engine::isolation::run_worker()
}

fn run() -> Result<()> {
    let args = Args::from_args();

    if !args.is_sandbox() {
        sandbox::fasten_seatbelt()?;
    }

    let config = Config::load_or_default()
        .context("Failed to load config")?;

    match args.subcommand {
        Some(SubCommand::Run(ref run)) => run_run(&args, run, config),
        Some(SubCommand::Sandbox(_)) => run_sandbox(),
        Some(SubCommand::Login(_)) => auth::run_login(&config),
        Some(SubCommand::Publish(ref publish)) => registry::run_publish(&args, publish, &config),
        Some(SubCommand::Install(ref install)) => registry::run_install(install, &config),
        Some(SubCommand::Search(ref search)) => registry::run_search(search, &config),
        Some(SubCommand::Completions(ref completions)) => complete::run_generate(completions),
        None => shell::run(&args, config),
    }
}

fn main() {
    env_logger::init();

    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        for cause in err.iter_chain().skip(1) {
            eprintln!("Because: {}", cause);
        }
        std::process::exit(1);
    }
}
