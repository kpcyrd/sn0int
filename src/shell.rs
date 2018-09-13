use errors::*;

use cmd::*;
use colored::Colorize;
use db::Database;
// use rand::prelude::*;
use rustyline;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
// use rustyline::{Cmd, CompletionType, Config, EditMode, Editor, Helper, KeyPress};
use rustyline::{CompletionType, Config, EditMode, Editor};
use shellwords;
use std::borrow::Cow::{self, Borrowed, Owned};
use psl::Psl;
// use std::io;
// use std::io::prelude::*;

use term::Prompt;
use worker;


#[derive(Debug)]
pub enum Command {
    Add,
    Back,
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


pub struct CmdCompleter;

impl CmdCompleter {

}

impl Completer for CmdCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        if line.contains(' ') || line.len() != pos {
            return Ok((0, vec![]));
        }

        let results: Vec<String> = Command::list_all().iter()
            .filter(|x| x.starts_with(line))
            .map(|x| x.to_string() + " ")
            .collect();

        Ok((0, results))
    }
}

// TODO: suggest rest of the line if only one possible completion
impl Hinter for CmdCompleter {
    #[inline]
    fn hint(&self, _line: &str, _pos: usize) -> Option<String> {
        // None
        match self.complete(_line, _pos) {
            Ok((_, mut cmds)) => if cmds.len() == 1 {
                // TODO: this fails if we complete a 2nd argument
                let hint = cmds.remove(0);
                Some(hint[_pos..].to_string())
            } else {
                None
            },
            Err(_) => None,
        }
    }
}

impl Highlighter for CmdCompleter {
    #[inline]
    fn highlight_prompt<'p>(&self, prompt: &'p str) -> Cow<'p, str> {
        Borrowed(prompt)
    }

    #[inline]
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[90m".to_owned() + hint + "\x1b[m")
    }
}

impl rustyline::Helper for CmdCompleter {}

pub struct Readline {
    rl: Editor<CmdCompleter>,
    prompt: Prompt,
    db: Database,
    psl: Psl,
}

impl Readline {
    pub fn new(db: Database, psl: Psl) -> Readline {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();
        let mut rl: Editor<CmdCompleter> = Editor::with_config(config);

        let h = CmdCompleter;
        rl.set_helper(Some(h));

        let prompt = Prompt::new(db.name().to_string());

        Readline {
            rl,
            prompt,
            db,
            psl,
        }
    }

    pub fn take_module(&mut self) -> Option<String> {
        self.prompt.module.take()
    }

    pub fn set_module(&mut self, module: String) {
        self.prompt.module = Some(module);
    }

    pub fn module(&self) -> Option<&String> {
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

                let action = match cmd[0].as_str() {
                    "add" => Some(Command::Add),
                    "back" => Some(Command::Back),
                    "run"  => Some(Command::Run),
                    "set"  => Some(Command::Set),
                    "show" => Some(Command::Show),
                    "switch_db" => Some(Command::SwitchDb),
                    "update" => Some(Command::Update),
                    "use"  => Some(Command::Use),
                    x => {
                        eprintln!("Error: unknown command: {:?}", x);
                        None
                    },
                };

                action.map(|x| (x, cmd))
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
                ?? modules
"#, "osint".green(), "recon".green(), "security".green());
}

pub fn run_once(rl: &mut Readline) -> Result<bool> {
    match rl.readline() {
        Some((Command::Add, args)) => add_cmd::run(rl, &args)?,
        Some((Command::Back, _)) => if rl.take_module().is_none() {
            return Ok(true);
        },
        Some((Command::Run, args)) => run_cmd::run(rl, &args)?,
        // TODO: show global settings
        // TODO: if module is some, show module settings
        // TODO: set jobs 25
        Some((Command::Set, _args)) => println!("set"),
        Some((Command::Show, args)) => show_cmd::run(rl, &args)?,
        Some((Command::SwitchDb, args)) => switch_db_cmd::run(rl, &args)?,
        Some((Command::Update, _args)) => {
            // TODO
            worker::spawn("Updating public suffix list");
            worker::spawn("Updating modules");
        },
        Some((Command::Use, args)) => use_cmd::run(rl, &args)?,
        Some((Command::Interrupt, _)) => return Ok(true),
        None => (),
    }

    Ok(false)
}

pub fn run() -> Result<()> {
    print_banner();

    // wait("checking tor status");

    // println!("\x1b[1m[\x1b[32m*\x1b[0;1m]\x1b[0m updating registry...");
    // wait("updating registry");

    let db = Database::establish("default")?;
    let psl = Psl::open_or_download()?;
    let mut rl = Readline::new(db, psl);

    loop {
        match run_once(&mut rl) {
            Ok(true) => break,
            Ok(_) => (),
            Err(err) => {
                eprintln!("{}", err);
                for cause in err.iter_chain().skip(1) {
                    eprintln!("Because: {}", cause);
                }
            },
        }
    }

    Ok(())
}
