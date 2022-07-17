use crate::errors::*;

use crate::args::Args;
use crate::blobs::{Blob, BlobStorage};
use crate::cmd::*;
use crate::config::Config;
use crate::db::ttl;
use crate::keyring::KeyRing;
use crate::worker::{self, VoidSender};
use colored::Colorize;
use crate::db::{self, Database};
use crate::engine::{Library, Module};
use crate::update::AutoUpdater;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::term::{self, Prompt};
use crate::paths;
use crate::psl::{Psl, PslReader};
use crate::lazy::Lazy;
use crate::workspaces::Workspace;

pub mod complete;
use self::complete::CmdCompleter;
pub mod readline;
use self::readline::{Readline, ReadlineError};


#[derive(Debug)]
pub enum Command {
    Activity,
    Add,
    Autonoscope,
    Autoscope,
    Back,
    Delete,
    Help,
    Keyring,
    Mod,
    Noscope,
    Pkg,
    Rescope,
    Run,
    Scope,
    Set,
    Select,
    Stats,
    Target,
    Use,
    Quickstart,
    Workspace,
    Cal,

    Exit,
    Quit,

    Interrupt,
    Exec(String),
}

impl Command {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Command::Activity => "activity",
            Command::Add => "add",
            Command::Autonoscope => "autonoscope",
            Command::Autoscope => "autoscope",
            Command::Back => "back",
            Command::Delete => "delete",
            Command::Exit => "exit",
            Command::Help => "help",
            Command::Keyring => "keyring",
            Command::Mod => "mod",
            Command::Noscope => "noscope",
            Command::Pkg => "pkg",
            Command::Rescope => "rescope",
            Command::Run => "run",
            Command::Scope => "scope",
            Command::Set => "set",
            Command::Select => "select",
            Command::Stats => "stats",
            Command::Target => "target",
            Command::Use => "use",
            Command::Quickstart => "quickstart",
            Command::Quit => "quit",
            Command::Workspace => "workspace",
            Command::Cal => "cal",
            Command::Interrupt => unreachable!(),
            Command::Exec(_) => unreachable!(),
        }
    }

    pub fn list_all() -> &'static [&'static str] {
        lazy_static! {
            static ref COMMANDS: Vec<&'static str> = vec![
                Command::Activity.as_str(),
                Command::Add.as_str(),
                Command::Autonoscope.as_str(),
                Command::Autoscope.as_str(),
                Command::Back.as_str(),
                Command::Delete.as_str(),
                Command::Exit.as_str(),
                Command::Help.as_str(),
                Command::Keyring.as_str(),
                Command::Noscope.as_str(),
                Command::Pkg.as_str(),
                Command::Rescope.as_str(),
                Command::Run.as_str(),
                Command::Scope.as_str(),
                Command::Set.as_str(),
                Command::Select.as_str(),
                Command::Stats.as_str(),
                Command::Target.as_str(),
                Command::Use.as_str(),
                Command::Quit.as_str(),
                Command::Workspace.as_str(),
                Command::Cal.as_str(),
            ];
        }

        COMMANDS.as_ref()
    }
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "activity" => Ok(Command::Activity),
            "add" => Ok(Command::Add),
            "autonoscope" => Ok(Command::Autonoscope),
            "autoscope" => Ok(Command::Autoscope),
            "back" => Ok(Command::Back),
            "delete" => Ok(Command::Delete),
            "exit" => Ok(Command::Exit),
            "help" => Ok(Command::Help),
            "keyring" => Ok(Command::Keyring),
            "mod" => Ok(Command::Mod),
            "noscope" => Ok(Command::Noscope),
            "pkg" => Ok(Command::Pkg),
            "rescope" => Ok(Command::Rescope),
            "run" => Ok(Command::Run),
            "scope" => Ok(Command::Scope),
            "set" => Ok(Command::Set),
            "select" => Ok(Command::Select),
            "stats" => Ok(Command::Stats),
            "target" => Ok(Command::Target),
            "use" => Ok(Command::Use),
            "quickstart" => Ok(Command::Quickstart),
            "quit" => Ok(Command::Quit),
            "workspace" => Ok(Command::Workspace),
            "cal" => Ok(Command::Cal),
            x => bail!("unknown command: {:?}, try \"help\"", x),
        }
    }
}

pub struct Shell<'a> {
    rl: Readline<CmdCompleter>,
    prompt: Prompt,
    db: Database,
    blobs: BlobStorage,
    psl: Lazy<PslReader, Arc<Psl>>,
    config: &'a Config,
    library: Library<'a>,
    keyring: KeyRing,
    // autonoscope: RuleSet,
    options: Option<HashMap<String, String>>,
    signal_register: Arc<SignalRegister>,
    cancel_twice: u8,
}

impl<'a> Shell<'a> {
    pub fn new(config: &'a Config, db: Database, blobs: BlobStorage, psl: PslReader, library: Library<'a>, keyring: KeyRing) -> Result<Shell<'a>> {
        let h = CmdCompleter::default();
        let rl = Readline::with(h)?;

        let prompt = Prompt::new(db.name().to_string());

        let mut rl = Shell {
            rl,
            prompt,
            db,
            blobs,
            psl: Lazy::from(psl),
            config,
            library,
            keyring,
            options: None,
            signal_register: Arc::new(SignalRegister::new()),
            cancel_twice: 0,
        };

        rl.reload_module_cache();
        rl.reload_keyring_cache();

        Ok(rl)
    }

    #[inline(always)]
    pub fn take_module(&mut self) -> Option<Module> {
        self.options = None;
        self.prompt.module.take()
    }

    #[inline(always)]
    pub fn set_module(&mut self, module: Module) {
        self.options = Some(HashMap::new());
        self.prompt.module = Some(module);
        // TODO: possibly refactor
        self.prompt.target = None;
    }

    #[inline(always)]
    pub fn module(&self) -> Option<&Module> {
        self.prompt.module.as_ref()
    }

    #[inline(always)]
    pub fn workspace(&self) -> &str {
        self.prompt.workspace.as_str()
    }

    #[inline(always)]
    pub fn options_mut(&mut self) -> Option<&mut HashMap<String, String>> {
        self.options.as_mut()
    }

    #[inline(always)]
    pub fn set_target(&mut self, target: Option<db::Filter>) {
        self.prompt.target = target;
    }

    #[inline(always)]
    pub fn target(&self) -> &Option<db::Filter> {
        &self.prompt.target
    }

    pub fn scoped_targets(&self) -> db::Filter {
        match &self.prompt.target {
            Some(filter) => filter.and_scoped(),
            _ => db::Filter::new("unscoped=0"),
        }
    }

    #[inline(always)]
    pub fn db(&self) -> &Database {
        &self.db
    }

    #[inline(always)]
    pub fn db_mut(&mut self) -> &mut Database {
        &mut self.db
    }

    #[inline(always)]
    pub fn set_db(&mut self, db: Database) {
        self.prompt.workspace = db.name().to_string();
        self.db = db;
    }

    #[inline(always)]
    pub fn blobs(&self) -> &BlobStorage {
        &self.blobs
    }

    #[inline(always)]
    pub fn set_blobstorage(&mut self, blobs: BlobStorage) {
        self.blobs = blobs;
    }

    #[inline(always)]
    pub fn psl(&mut self) -> Result<&Arc<Psl>> {
        Ok(self.psl.get()?)
    }

    #[inline(always)]
    pub fn config(&self) -> &Config {
        self.config
    }

    #[inline(always)]
    pub fn library(&self) -> &Library {
        &self.library
    }

    #[inline(always)]
    pub fn library_mut(&mut self) -> &mut Library<'a> {
        &mut self.library
    }

    #[inline(always)]
    pub fn keyring(&self) -> &KeyRing {
        &self.keyring
    }

    #[inline(always)]
    pub fn keyring_mut(&mut self) -> &mut KeyRing {
        &mut self.keyring
    }

    pub fn readline(&mut self) -> Option<(Command, Vec<String>)> {
        let readline = self.rl.readline(&self.prompt.to_string());

        if readline.is_ok() {
            self.cancel_twice = 0;
        }

        match readline {
            Ok(line) => {
                self.cancel_twice = 0;
                if line.is_empty() {
                    None
                } else {
                    debug!("Readline returned {:?}", line);

                    self.rl.add_history_entry(line.as_str());

                    if line.starts_with('#') {
                        return None;
                    }

                    if let Some(cmd) = line.strip_prefix('!') {
                        return Some((Command::Exec(cmd.to_string()), vec![]));
                    }

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
            }
            Err(ReadlineError::Interrupted) => {
                // ^C
                self.cancel_twice += 1;

                if self.cancel_twice > 1 {
                    Some((Command::Interrupt, vec![]))
                } else {
                    None
                }
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

    pub fn reload_modules(&mut self) -> Result<()> {
        let current = self.take_module()
                        .map(|m| m.canonical());

        self.library_mut().reload_modules()?;
        self.reload_module_cache();

        if let Some(module) = current {
            if let Ok(module) = self.library().get(&module) {
                let module = module.clone();
                self.set_module(module);
            }
        }

        Ok(())
    }

    pub fn reload_module_cache(&mut self) {
        if let Some(helper) = self.rl.helper_mut() {
            helper.modules.clear();
            for module in self.library.variants() {
                helper.modules.push(module);
            }
        }
    }

    pub fn reload_keyring_cache(&mut self) {
        let keys = self.keyring().list().iter()
            .map(|k| k.to_string())
            .collect();

        if let Some(helper) = self.rl.helper_mut() {
            helper.keyring = keys;
        }
    }

    pub fn load_history(&mut self) -> Result<()> {
        self.rl.load_history(&paths::history_path()?)
    }

    pub fn save_history(&mut self) -> Result<()> {
        self.rl.save_history(&paths::history_path()?)
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

    #[inline(always)]
    pub fn signal_register(&self) -> &Arc<SignalRegister> {
        &self.signal_register
    }

    pub fn store_blob(&self, tx: VoidSender, blob: &Blob) {
        let result = self.blobs.save(blob)
            .map_err(|err| err.to_string());
        tx.send(result).unwrap();
    }
}

pub struct SignalRegister(AtomicUsize);

impl Default for SignalRegister {
    fn default() -> Self {
        Self::new()
    }
}

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

#[inline(always)]
fn cmd<T: Cmd>(rl: &mut Shell, args: &[String]) -> Result<()> {
    T::run_str(rl, args)
}

pub fn run_once(rl: &mut Shell) -> Result<bool> {
    let line = rl.readline();
    debug!("Received line: {:?}", line);
    match line {
        Some((Command::Activity, args)) => cmd::<activity_cmd::Args>(rl, &args)?,
        Some((Command::Add, args)) => cmd::<add_cmd::Args>(rl, &args)?,
        Some((Command::Autonoscope, args)) => cmd::<autonoscope_cmd::Args>(rl, &args)?,
        Some((Command::Autoscope, args)) => cmd::<autoscope_cmd::Args>(rl, &args)?,
        Some((Command::Back, _)) => if rl.take_module().is_none() {
            return Ok(true);
        },
        Some((Command::Delete, args)) => delete_cmd::run(rl, &args)?,
        Some((Command::Help, args)) => help_cmd::run(rl, &args)?,
        Some((Command::Keyring, args)) => keyring_cmd::run(rl, &args)?,
        Some((Command::Mod, args)) => {
            term::warn("The \x1b[1mmod\x1b[0m command is deprecated, use \x1b[1mpkg\x1b[0m");
            cmd::<pkg_cmd::ArgsInteractive>(rl, &args)?
        },
        Some((Command::Noscope, args)) => noscope_cmd::run(rl, &args)?,
        Some((Command::Pkg, args)) => cmd::<pkg_cmd::ArgsInteractive>(rl, &args)?,
        Some((Command::Rescope, args)) => cmd::<rescope_cmd::Args>(rl, &args)?,
        Some((Command::Run, args)) => cmd::<run_cmd::Args>(rl, &args)?,
        Some((Command::Scope, args)) => scope_cmd::run(rl, &args)?,
        Some((Command::Set, args)) => set_cmd::run(rl, &args)?,
        Some((Command::Select, args)) => cmd::<select_cmd::Args>(rl, &args)?,
        Some((Command::Stats, args)) => cmd::<stats_cmd::Args>(rl, &args)?,
        Some((Command::Target, args)) => target_cmd::run(rl, &args)?,
        Some((Command::Use, args)) => use_cmd::run(rl, &args)?,
        Some((Command::Quickstart, args)) => quickstart_cmd::run(rl, &args)?,
        Some((Command::Workspace, args)) => cmd::<workspace_cmd::Args>(rl, &args)?,
        Some((Command::Cal, args)) => cmd::<cal_cmd::Args>(rl, &args)?,

        Some((Command::Exit, _)) => return Ok(true),
        Some((Command::Quit, _)) => return Ok(true),
        Some((Command::Interrupt, _)) => return Ok(true),
        Some((Command::Exec(cmd), _)) => {
            shell_exec(&cmd, rl.workspace())?;
        },
        None => (),
    }

    Ok(false)
}

pub fn init<'a>(args: &Args, config: &'a Config, verbose_init: bool) -> Result<Shell<'a>> {
    let workspace = match args.workspace {
        Some(ref workspace) => workspace.clone(),
        None => Workspace::from_str("default").unwrap(),
    };

    workspace.migrate()?;

    let blobs = BlobStorage::workspace(&workspace)?;
    let db = if verbose_init {
        Database::establish(workspace)?
    } else {
        Database::establish_quiet(workspace)?
    };

    let cache_dir = paths::cache_dir()?;
    let psl = PslReader::open_or_download(&cache_dir,
            |cb| worker::spawn_fn("Downloading public suffix list", cb, false))
        .context("Failed to download public suffix list")?;
    let library = Library::new(verbose_init, config)?;
    let keyring = KeyRing::init()?;

    if verbose_init && library.list().is_empty() {
        term::success("No modules found, run \x1b[1mpkg quickstart\x1b[0m to install default modules");
        term::success("New to sn0int? Follow https://sn0int.rtfd.io/en/stable/usage.html");
    }

    let autoupdate = AutoUpdater::load()?;
    if autoupdate.outdated() > 0 {
        term::warn(&format!("{} modules are outdated, run: \x1b[1mpkg update\x1b[0m", autoupdate.outdated()));
    }
    autoupdate.check_background(config, library.list());

    let mut rl = Shell::new(config, db, blobs, psl, library, keyring)?;

    ttl::reap_expired(&mut rl)?;

    Ok(rl)
}

pub fn shell_exec(cmd: &str, workspace: &str) -> Result<()> {
    use std::process::Command;

    let sh = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    Command::new(sh.0)
            .arg(sh.1)
            .arg(cmd)
            .env("SN0INT_WORKSPACE", workspace)
            .spawn()
            .context("Failed to execute process")?
            .wait()?;

    Ok(())
}

pub fn run(args: &Args, config: &Config) -> Result<()> {
    print_banner();

    let mut rl = init(args, config, true)?;
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
