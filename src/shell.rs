use errors::*;

use args::Args;
use cmd::*;
use complete::CmdCompleter;
use colored::Colorize;
use db::Database;
use engine::{Engine, Module};
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, EditMode, Editor};
use shellwords;
use std::str::FromStr;
use term;
use psl::Psl;

use term::Prompt;


#[derive(Debug)]
pub enum Command {
    Add,
    Back,
    List,
    ReloadModules,
    Run,
    Set,
    Show,
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
            Command::List => "list",
            Command::ReloadModules => "reload_modules",
            Command::Run => "run",
            Command::Set => "set",
            Command::Show => "show",
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
                Command::List.as_str(),
                Command::ReloadModules.as_str(),
                Command::Run.as_str(),
                Command::Set.as_str(),
                Command::Show.as_str(),
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
            "list" => Ok(Command::List),
            "reload_modules"  => Ok(Command::ReloadModules),
            "run"  => Ok(Command::Run),
            "set"  => Ok(Command::Set),
            "show" => Ok(Command::Show),
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
    engine: Engine,
}

impl Readline {
    pub fn new(db: Database, psl: Psl, engine: Engine) -> Readline {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();
        let mut rl: Editor<CmdCompleter> = Editor::with_config(config);

        let h = CmdCompleter::default();
        rl.set_helper(Some(h));

        let prompt = Prompt::new(db.name().to_string());

        let mut rl = Readline {
            rl,
            prompt,
            db,
            psl,
            engine,
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
                self.rl.add_history_entry(line.as_ref());

                let cmd = match shellwords::split(&line) {
                    Ok(cmd) => cmd,
                    Err(err) => {
                        eprintln!("Error: {:?}", err);
                        return None;
                    },
                };

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
        Some((Command::List, args)) => list_cmd::run(rl, &args)?,
        Some((Command::ReloadModules, args)) => reload_modules_cmd::run(rl, &args)?,
        Some((Command::Run, args)) => run_cmd::run(rl, &args)?,
        // TODO: show global settings
        // TODO: if module is some, show module settings
        // TODO: set jobs 25
        Some((Command::Set, _args)) => println!("set"),
        Some((Command::Show, args)) => show_cmd::run(rl, &args)?,
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

pub fn run(args: &Args) -> Result<()> {
    print_banner();

    // wait("checking tor status");

    // println!("\x1b[1m[\x1b[32m*\x1b[0;1m]\x1b[0m updating registry...");
    // wait("updating registry");

    // TODO: enforce valid characters for workspace name
    let workspace = match args.workspace {
        Some(ref workspace) => workspace.as_str(),
        None => "default",
    };

    let db = Database::establish(workspace)?;
    let psl = Psl::open_or_download()?;
    let engine = Engine::new()?;
    let mut rl = Readline::new(db, psl, engine);

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

    Ok(())
}
