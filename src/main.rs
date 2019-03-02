#![warn(unused_extern_crates)]
#[macro_use] extern crate failure;
#[macro_use] extern crate log;

use env_logger::{self, Env};
use sn0int::args::{self, Args, SubCommand};
use sn0int::auth;
use sn0int::cmd;
use sn0int::complete;
use sn0int::config::Config;
use sn0int::errors::*;
use sn0int::engine::{self, Module};
use sn0int::geoip::{GeoIP, AsnDB, Maxmind};
use sn0int::options::Opt;
use sn0int::psl::Psl;
use sn0int::registry;
use sn0int::sandbox;
use sn0int::shell;
use structopt::StructOpt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;


fn run_run(gargs: &Args, args: &args::Run, config: Config) -> Result<()> {
    let mut rl = shell::init(gargs, config, false)?;

    if let Some(module) = &args.module {
        let module = rl.engine().get(&module)?.clone();
        rl.set_module(module);
    } else if let Some(file) = &args.file {
        let path = Path::new(file);

        let filename = path.file_stem()
            .ok_or(format_err!("Failed to decode filename"))?
            .to_str()
            .ok_or(format_err!("Failed to decode filename"))?;

        let module = Module::load(&path.to_path_buf(), "anonymous", &filename, true)
            .context(format!("Failed to parse {:?}", file))?;
        rl.set_module(module);
    } else {
        bail!("At least one module or file need to be provided");
    }

    cmd::run_cmd::execute(&mut rl, args.into(), Opt::collect(&args.options))
}

fn run_sandbox() -> Result<()> {
    let geoip = GeoIP::open_into_buf()?;
    let asn = AsnDB::open_into_buf()?;
    let psl = Psl::open_into_string()?;

    sandbox::init()
        .context("Failed to init sandbox")?;
    engine::isolation::run_worker(geoip, asn, &psl)
}

fn run_cmd<T: cmd::Cmd>(gargs: &Args, args: &T, config: Config) -> Result<()> {
    let mut rl = shell::init(gargs, config, false)?;
    args.run(&mut rl)
}

fn run_new(_gargs: &Args, args: &args::New) -> Result<()> {
    let boilerplate = b"-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    -- TODO your code here
end
";

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args.path)?;
    file.write_all(boilerplate)?;

    Ok(())
}

fn run() -> Result<()> {
    let args = Args::from_args();

    if !args.is_sandbox() {
        sandbox::fasten_seatbelt()?;
    }

    let config = Config::load_or_default()
        .context("Failed to load config")?;

    debug!("Loaded config: {:?}", config);

    match args.subcommand {
        Some(SubCommand::Run(ref run)) => run_run(&args, run, config),
        Some(SubCommand::Sandbox(_)) => run_sandbox(),
        Some(SubCommand::Login(_)) => auth::run_login(&config),
        Some(SubCommand::New(ref new)) => run_new(&args, new),
        Some(SubCommand::Publish(ref publish)) => registry::run_publish(&args, publish, &config),
        Some(SubCommand::Install(ref install)) => registry::run_install(install, &config),
        Some(SubCommand::Search(ref search)) => registry::run_search(search, &config),
        Some(SubCommand::Select(ref select)) => run_cmd(&args, select, config),
        Some(SubCommand::Completions(ref completions)) => complete::run_generate(completions),
        None => shell::run(&args, config),
    }
}

fn main() {
    env_logger::init_from_env(Env::default()
        .default_filter_or("off"));

    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        for cause in err.iter_chain().skip(1) {
            eprintln!("Because: {}", cause);
        }
        std::process::exit(1);
    }
}
