use env_logger::Env;
use sn0int::args::{self, Args, SubCommand};
use sn0int::auth;
use sn0int::cmd::{self, LiteCmd};
use sn0int::cmd::run_cmd::Params;
use sn0int::config::Config;
use sn0int::db;
use sn0int::errors::*;
use sn0int::engine::Module;
use sn0int::geoip::{GeoIP, AsnDB, Maxmind};
use sn0int::ipc;
use sn0int::options::Opt;
use sn0int::paths;
use sn0int::psl::PslReader;
use sn0int::registry;
use sn0int::repl;
use sn0int::sandbox;
use sn0int::shell::{self, complete};
use structopt::StructOpt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

fn run_run(gargs: &Args, args: &args::Run, config: &Config) -> Result<()> {
    let mut rl = shell::init(gargs, config, false)?;

    let module = args.run.module.as_ref()
        .ok_or_else(|| format_err!("Module is required"))?;

    let module = if args.file {
        let path = Path::new(&module);

        let filename = path.file_stem()
            .ok_or_else(|| format_err!("Failed to decode filename"))?
            .to_str()
            .ok_or_else(|| format_err!("Failed to decode filename"))?;

        Module::load(path, "anonymous", filename, true)
            .context(format!("Failed to parse {:?}", path))?
    } else {
        rl.library().get(module)?
            .clone()
    };

    rl.set_module(module);

    if let Some(target) = &args.target {
        let target = shellwords::split(target)
            .map_err(|_| format_err!("Failed to parse target quotes"))?;
        let target = db::Filter::parse(&target)?;
        rl.set_target(Some(target));
    }

    let mut params = Params::from(args);
    // The module was already set and loaded
    params.module = None;

    if args.dump_sandbox_init_msg {
        cmd::run_cmd::dump_sandbox_init_msg(&mut rl, params, Opt::collect(&args.options))
    } else {
        cmd::run_cmd::execute(&mut rl, params, Opt::collect(&args.options))
    }
}

fn run_sandbox() -> Result<()> {
    let cache_dir = paths::cache_dir()?;
    let geoip = GeoIP::try_open_reader(&cache_dir)?;
    let asn = AsnDB::try_open_reader(&cache_dir)?;
    let psl = PslReader::open(&cache_dir)?;

    sandbox::init()
        .context("Failed to init sandbox")?;
    ipc::child::run(geoip, asn, psl)
}

fn run_cmd<T: cmd::Cmd>(gargs: &Args, args: T, config: &Config) -> Result<()> {
    let mut rl = shell::init(gargs, config, false)?;
    args.run(&mut rl)
}

fn run_new(_gargs: &Args, args: &args::New) -> Result<()> {
    // TODO: Add `-- Author: anonymous`
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
    let mut args = Args::from_args();

    if !args.is_sandbox() {
        sandbox::fasten_seatbelt()?;
    }

    let config = Config::load_or_default()
        .context("Failed to load config")?;

    debug!("Loaded config: {:?}", config);

    match args.subcommand.take() {
        Some(SubCommand::Run(run)) => run_run(&args, &run, &config),
        Some(SubCommand::Sandbox(_)) => run_sandbox(),
        Some(SubCommand::Login(_)) => auth::run_login(&config),
        Some(SubCommand::New(new)) => run_new(&args, &new),
        Some(SubCommand::Publish(publish)) => registry::run_publish(&args, &publish, &config),
        // TODO: deprecate
        Some(SubCommand::Install(install)) => cmd::pkg_cmd::Args {
            subcommand: cmd::pkg_cmd::SubCommand::Install(install),
        }.run(&config),
        // TODO: deprecate
        Some(SubCommand::Search(search)) => cmd::pkg_cmd::Args {
            subcommand: cmd::pkg_cmd::SubCommand::Search(search),
        }.run(&config),
        Some(SubCommand::Pkg(pkg)) => pkg.run(&config),
        Some(SubCommand::Add(add)) => run_cmd(&args, add, &config),
        Some(SubCommand::Select(select)) => run_cmd(&args, select, &config),
        Some(SubCommand::Delete(delete)) => run_cmd(&args, delete, &config),
        Some(SubCommand::Activity(activity)) => run_cmd(&args, activity, &config),
        Some(SubCommand::Scope(scope)) => run_cmd(&args, scope, &config),
        Some(SubCommand::Noscope(noscope)) => run_cmd(&args, noscope, &config),
        Some(SubCommand::Autoscope(autoscope)) => run_cmd(&args, autoscope, &config),
        Some(SubCommand::Autonoscope(autonoscope)) => run_cmd(&args, autonoscope, &config),
        Some(SubCommand::Rescope(rescope)) => run_cmd(&args, rescope, &config),
        Some(SubCommand::Workspace(workspace)) => workspace.run(&config),
        Some(SubCommand::Fsck(fsck)) => run_cmd(&args, fsck, &config),
        Some(SubCommand::Export(export)) => run_cmd(&args, export, &config),
        Some(SubCommand::Cal(cal)) => run_cmd(&args, cal, &config),
        Some(SubCommand::Notify(notify)) => run_cmd(&args, notify, &config),
        Some(SubCommand::Stats(stats)) => run_cmd(&args, stats, &config),
        Some(SubCommand::Repl) => repl::run(&config),
        Some(SubCommand::Paths) => paths::run(&config),
        Some(SubCommand::Completions(completions)) => complete::run_generate(&completions),
        None => shell::run(&args, &config),
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
