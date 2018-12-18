use crate::errors::*;

use crate::accesskey::KeyStore;
use crate::args::Args;
use crate::cmd::*;
use crate::complete::CmdCompleter;
use crate::config::Config;
use colored::Colorize;
use ctrlc;
use crate::db::{self, Database};
use crate::engine::{Engine, Module};
use crate::geoip::{GeoIP, AsnDB, Maxmind};
use rustyline::error::ReadlineError;
use rustyline::{self, CompletionType, EditMode, Editor};
use shellwords;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::term::{self, Prompt};
use crate::paths;
use crate::psl::Psl;
use crate::workspaces::Workspace;


#[derive(Debug)]
pub enum Command {
    AccessKey,
    Add,
    Back,
    Delete,
    Mod,
    Noscope,
    Run,
    Scope,
    Set,
    Select,
    Target,
    Use,
    Quickstart,
    Workspace,

    Interrupt,
}

impl Command {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Command::AccessKey => "accesskey",
            Command::Add => "add",
            Command::Back => "back",
            Command::Delete => "delete",
            Command::Mod => "mod",
            Command::Noscope => "noscope",
            Command::Run => "run",
            Command::Scope => "scope",
            Command::Set => "set",
            Command::Select => "select",
            Command::Target => "target",
            Command::Use => "use",
            Command::Quickstart => "quickstart",
            Command::Workspace => "workspace",
            Command::Interrupt => unreachable!(),
        }
    }

    pub fn list_all() -> &'static [&'static str] {
        lazy_static! {
            static ref COMMANDS: Vec<&'static str> = vec![
                Command::AccessKey.as_str(),
                Command::Add.as_str(),
                Command::Back.as_str(),
                Command::Delete.as_str(),
                Command::Mod.as_str(),
                Command::Noscope.as_str(),
                Command::Run.as_str(),
                Command::Scope.as_str(),
                Command::Set.as_str(),
                Command::Select.as_str(),
                Command::Workspace.as_str(),
                Command::Target.as_str(),
                Command::Use.as_str(),
                Command::Quickstart.as_str(),
            ];
        }

        COMMANDS.as_ref()
    }
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "accesskey" => Ok(Command::AccessKey),
            "add" => Ok(Command::Add),
            "back" => Ok(Command::Back),
            "delete" => Ok(Command::Delete),
            "mod" => Ok(Command::Mod),
            "noscope" => Ok(Command::Noscope),
            "run"  => Ok(Command::Run),
            "scope"  => Ok(Command::Scope),
            "set"  => Ok(Command::Set),
            "select" => Ok(Command::Select),
            "target"  => Ok(Command::Target),
            "use"  => Ok(Command::Use),
            "quickstart"  => Ok(Command::Quickstart),
            "workspace" => Ok(Command::Workspace),
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
    keystore: KeyStore,
    signal_register: Arc<SignalRegister>,
}

impl Readline {
    pub fn new(config: Config, db: Database, psl: Psl, engine: Engine, keystore: KeyStore) -> Readline {
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
            keystore,
            signal_register: Arc::new(SignalRegister::new()),
        };

        rl.reload_module_cache();

        rl
    }

    pub fn take_module(&mut self) -> Option<Module> {
        self.prompt.module.take()
    }

    pub fn set_module(&mut self, module: Module) {
        self.prompt.module = Some(module);
        // TODO: possibly refactor
        self.prompt.target = None;
    }

    pub fn module(&self) -> Option<&Module> {
        self.prompt.module.as_ref()
    }

    pub fn set_target(&mut self, target: Option<db::Filter>) {
        self.prompt.target = target;
    }

    pub fn target(&self) -> &Option<db::Filter> {
        &self.prompt.target
    }

    pub fn scoped_targets(&self) -> db::Filter {
        match &self.prompt.target {
            Some(filter) => filter.and_scoped(),
            _ => db::Filter::new("unscoped=0"),
        }
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

    pub fn keystore(&self) -> &KeyStore {
        &self.keystore
    }

    pub fn keystore_mut(&mut self) -> &mut KeyStore {
        &mut self.keystore
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

                match Command::from_str(&cmd[0]) {
                    Ok(x) => Some((x, cmd)),
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        None
                    },
                }
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
            let prev = ctr.add_ctrlc();
            if prev == 1 {
                ::std::process::exit(0);
            }
        }).map_err(Error::from)
    }

    pub fn signal_register(&self) -> &Arc<SignalRegister> {
        &self.signal_register
    }
}

pub struct SignalRegister(AtomicUsize);

impl SignalRegister {
    pub fn new() -> SignalRegister {
        SignalRegister(AtomicUsize::new(1))
    }

    pub fn catch_ctrl(&self) {
        self.0.store(0, Ordering::SeqCst);
    }

    pub fn add_ctrlc(&self) -> usize {
        self.0.fetch_add(1, Ordering::SeqCst)
    }

    pub fn ctrlc_received(&self) -> bool {
        self.0.load(Ordering::SeqCst) == 1
    }

    pub fn reset_ctrlc(&self) {
        self.0.store(1, Ordering::SeqCst);
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
      {}
"#, "osint".green(), "recon".green(), "security".green(),
"irc.hackint.org:6697/#sn0int".green());
}

pub fn run_once(rl: &mut Readline) -> Result<bool> {
    let line = rl.readline();
    debug!("Received line: {:?}", line);
    match line {
        Some((Command::AccessKey, args)) => accesskey_cmd::run(rl, &args)?,
        Some((Command::Add, args)) => add_cmd::run(rl, &args)?,
        Some((Command::Back, _)) => if rl.take_module().is_none() {
            return Ok(true);
        },
        Some((Command::Delete, args)) => delete_cmd::run(rl, &args)?,
        Some((Command::Mod, args)) => mod_cmd::run(rl, &args)?,
        Some((Command::Noscope, args)) => noscope_cmd::run(rl, &args)?,
        Some((Command::Run, args)) => run_cmd::run(rl, &args)?,
        Some((Command::Scope, args)) => scope_cmd::run(rl, &args)?,
        // TODO: show global settings
        // TODO: if module is some, show module settings
        // TODO: set jobs 25
        Some((Command::Set, _args)) => println!("set"),
        Some((Command::Select, args)) => select_cmd::run(rl, &args)?,
        Some((Command::Target, args)) => target_cmd::run(rl, &args)?,
        Some((Command::Use, args)) => use_cmd::run(rl, &args)?,
        Some((Command::Quickstart, args)) => quickstart_cmd::run(rl, &args)?,
        Some((Command::Workspace, args)) => workspace_cmd::run(rl, &args)?,
        Some((Command::Interrupt, _)) => return Ok(true),
        None => (),
    }

    Ok(false)
}

pub fn init(args: &Args, config: Config) -> Result<Readline> {
    let workspace = match args.workspace {
        Some(ref workspace) => workspace.clone(),
        None => Workspace::from_str("default").unwrap(),
    };

    let db = Database::establish(workspace)?;
    let psl = Psl::open_or_download()?;
    let _geoip = GeoIP::open_or_download()?;
    let _asndb = AsnDB::open_or_download()?;
    let engine = Engine::new()?;
    let keystore = KeyStore::init()?;

    if engine.list().is_empty() {
        term::success("No modules found, run quickstart to install default modules");
    }

    let rl = Readline::new(config, db, psl, engine, keystore);

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
