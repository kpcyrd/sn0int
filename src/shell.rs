use errors::*;

use args::Args;
use cmd::*;
use complete::CmdCompleter;
use config::Config;
use colored::Colorize;
use ctrlc;
use db::Database;
use engine::{Engine, Module};
use geoip::{GeoIP, AsnDB, Maxmind};
use rustyline::error::ReadlineError;
use rustyline::{self, CompletionType, EditMode, Editor};
use shellwords;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use term;
use paths;
use psl::Psl;

use term::Prompt;


#[derive(Debug)]
pub enum Command {
    Add,
    Back,
    Mod,
    Noscope,
    Run,
    Scope,
    Set,
    Select,
    SwitchDb,
    Update,
    Use,

    Interrupt,
}

impl Command {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Command::Add => "add",
            Command::Back => "back",
            Command::Mod => "mod",
            Command::Noscope => "noscope",
            Command::Run => "run",
            Command::Scope => "scope",
            Command::Set => "set",
            Command::Select => "select",
            Command::SwitchDb => "switch_db",
            Command::Update => "update",
            Command::Use => "use",
            Command::Interrupt => unreachable!(),
        }
    }

    pub fn list_all() -> &'static [&'static str] {
        lazy_static! {
            static ref COMMANDS: Vec<&'static str> = vec![
                Command::Add.as_str(),
                Command::Back.as_str(),
                Command::Mod.as_str(),
                Command::Noscope.as_str(),
                Command::Run.as_str(),
                Command::Scope.as_str(),
                Command::Set.as_str(),
                Command::Select.as_str(),
                Command::SwitchDb.as_str(),
                Command::Update.as_str(),
                Command::Use.as_str(),
            ];
        }

        COMMANDS.as_ref()
    }
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "add" => Ok(Command::Add),
            "back" => Ok(Command::Back),
            "mod" => Ok(Command::Mod),
            "noscope" => Ok(Command::Noscope),
            "run"  => Ok(Command::Run),
            "scope"  => Ok(Command::Scope),
            "set"  => Ok(Command::Set),
            "select" => Ok(Command::Select),
            "switch_db" => Ok(Command::SwitchDb),
            "update" => Ok(Command::Update),
            "use"  => Ok(Command::Use),
            x => bail!("unknown command: {:?}", x),
        }
    }
}

pub struct Readline {
    rl: Editor<CmdCompleter>,
    prompt: Prompt,
    db: Database,
    psl: Psl,
    config: Config,
    engine: Engine,
    signal_register: Arc<AtomicUsize>,
}

impl Readline {
    pub fn new(config: Config, db: Database, psl: Psl, engine: Engine) -> Readline {
        let rl_config = rustyline::Config::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();
        let mut rl: Editor<CmdCompleter> = Editor::with_config(rl_config);

        let h = CmdCompleter::default();
        rl.set_helper(Some(h));

        let prompt = Prompt::new(db.name().to_string());

        let mut rl = Readline {
            rl,
            prompt,
            db,
            psl,
            config,
            engine,
            signal_register: Arc::new(AtomicUsize::new(1)),
        };

        rl.reload_module_cache();

        rl
    }

    pub fn take_module(&mut self) -> Option<Module> {
        self.prompt.module.take()
    }

    pub fn set_module(&mut self, module: Module) {
        self.prompt.module = Some(module);
    }

    pub fn module(&self) -> Option<&Module> {
        self.prompt.module.as_ref()
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn set_db(&mut self, db: Database) {
        self.prompt.workspace = db.name().to_string();
        self.db = db;
    }

    pub fn psl(&self) -> &Psl {
        &self.psl
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    pub fn readline(&mut self) -> Option<(Command, Vec<String>)> {
        let readline = self.rl.readline(&self.prompt.to_string());
        match readline {
            Ok(ref line) if line.is_empty() => None,
            Ok(line) => {
                debug!("Readline returned {:?}", line);

                self.rl.add_history_entry(line.as_str());

                let cmd = match shellwords::split(&line) {
                    Ok(cmd) => cmd,
                    Err(err) => {
                        eprintln!("Error: {:?}", err);
                        return None;
                    },
                };
                debug!("shellwords returned {:?}", cmd);

                if cmd.is_empty() {
                    return None;
                }

                Command::from_str(&cmd[0]).ok()
                    .map(|x| (x, cmd))
            }
            Err(ReadlineError::Interrupted) => {
                // ^C
                Some((Command::Interrupt, vec![]))
            }
            Err(ReadlineError::Eof) => {
                // ^D
                Some((Command::Back, vec![]))
            }
            Err(err) => {
                println!("Error: {:?}", err);
                Some((Command::Interrupt, vec![]))
            }
        }
    }

    pub fn reload_module_cache(&mut self) {
        if let Some(helper) = self.rl.helper_mut() {
            helper.modules.clear();
            for module in self.engine.variants() {
                helper.modules.push(module);
            }
        }
    }

    pub fn load_history(&mut self) -> Result<()> {
        self.rl.load_history(&paths::history_path()?)
            .map_err(Error::from)
    }

    pub fn save_history(&self) -> Result<()> {
        self.rl.save_history(&paths::history_path()?)
            .map_err(Error::from)
    }

    pub fn set_signal_handler(&self) -> Result<()> {
        let ctr = self.signal_register.clone();
        ctrlc::set_handler(move || {
            // it seems this handler is only executed if rustyline is not active
            // this sends a SIGINT to all child processes (if any), terminating the worker for us
            //
            // by default ctr has a value of 1, if we expect a situation where we want to catch ctrlc
            // we set it to 0. if we receive ctrl+c we increase it by one and terminate if the ctr has a
            // value of 2 afterwards. If we don't want to catch ctrlcs anymore we set the ctr back to 1.
            // This is important so we can still terminate the process while we are reading input from stdin,
            // eg while waiting for input during `add domain`.
            let prev = ctr.fetch_add(1, Ordering::SeqCst);
            if prev == 1 {
                ::std::process::exit(0);
            }
        }).map_err(Error::from)
    }

    pub fn catch_ctrl(&self) {
        self.signal_register.store(0, Ordering::SeqCst);
    }

    pub fn ctrlc_received(&self) -> bool {
        self.signal_register.load(Ordering::SeqCst) == 1
    }

    pub fn reset_ctrlc(&self) {
        self.signal_register.store(1, Ordering::SeqCst);
    }
}


#[inline]
pub fn print_banner() {
    println!(r#"
                   ___/           .
     ____ , __   .'  /\ ` , __   _/_
    (     |'  `. |  / | | |'  `.  |
    `--.  |    | |,'  | | |    |  |
   \___.' /    | /`---' / /    |  \__/

        {} | {} | {}
"#, "osint".green(), "recon".green(), "security".green());
}

pub fn run_once(rl: &mut Readline) -> Result<bool> {
    let line = rl.readline();
    debug!("Received line: {:?}", line);
    match line {
        Some((Command::Add, args)) => add_cmd::run(rl, &args)?,
        Some((Command::Back, _)) => if rl.take_module().is_none() {
            return Ok(true);
        },
        Some((Command::Mod, args)) => mod_cmd::run(rl, &args)?,
        Some((Command::Noscope, args)) => noscope_cmd::run(rl, &args)?,
        Some((Command::Run, args)) => run_cmd::run(rl, &args)?,
        Some((Command::Scope, args)) => scope_cmd::run(rl, &args)?,
        // TODO: show global settings
        // TODO: if module is some, show module settings
        // TODO: set jobs 25
        Some((Command::Set, _args)) => println!("set"),
        Some((Command::Select, args)) => select_cmd::run(rl, &args)?,
        Some((Command::SwitchDb, args)) => switch_db_cmd::run(rl, &args)?,
        Some((Command::Update, _args)) => {
            // TODO
            // worker::spawn("Updating public suffix list");
            // worker::spawn("Updating modules");
        },
        Some((Command::Use, args)) => use_cmd::run(rl, &args)?,
        Some((Command::Interrupt, _)) => return Ok(true),
        None => (),
    }

    Ok(false)
}

pub fn init(args: &Args, config: Config) -> Result<Readline> {
    // TODO: enforce valid characters for workspace name
    let workspace = match args.workspace {
        Some(ref workspace) => workspace.as_str(),
        None => "default",
    };

    let db = Database::establish(workspace)?;
    let psl = Psl::open_or_download()?;
    let _geoip = GeoIP::open_or_download()?;
    let _asndb = AsnDB::open_or_download()?;
    let engine = Engine::new()?;
    let rl = Readline::new(config, db, psl, engine);

    Ok(rl)
}

pub fn run(args: &Args, config: Config) -> Result<()> {
    print_banner();

    let mut rl = init(args, config)?;
    rl.load_history().ok();

    rl.set_signal_handler()
        .context("Failed to set signal handler")?;

    loop {
        match run_once(&mut rl) {
            Ok(true) => break,
            Ok(_) => (),
            Err(err) => {
                term::error(&err.to_string());
                for cause in err.iter_chain().skip(1) {
                    eprintln!("Because: {}", cause);
                }
            },
        }
    }

    rl.save_history()?;

    Ok(())
}
