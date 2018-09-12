use errors::*;

use colored::Colorize;
use diesel::prelude::*;
use migrations;
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
    Update,
    Use(Vec<String>),

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
            Command::Update => "update",
            Command::Use(_) => "use",
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
                Command::Update.as_str(),
                Command::Use(Vec::new()).as_str(),
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
}

impl Readline {
    pub fn new() -> Readline {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();
        let mut rl: Editor<CmdCompleter> = Editor::with_config(config);

        let h = CmdCompleter;
        rl.set_helper(Some(h));

        let prompt = Prompt::new();

        Readline {
            rl,
            prompt,
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

    pub fn next(&mut self) -> Option<Command> {
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

                match cmd[0].as_str() {
                    "add" => Some(Command::Add),
                    "back" => Some(Command::Back),
                    "run"  => Some(Command::Run),
                    "set"  => Some(Command::Set),
                    "show" => Some(Command::Show),
                    "update" => Some(Command::Update),
                    "use"  => Some(Command::Use(cmd[1..].to_vec())),
                    x => {
                        eprintln!("Error: unknown command: {:?}", x);
                        return None;
                    },
                }
            }
            Err(ReadlineError::Interrupted) => {
                // ^C
                Some(Command::Interrupt)
            }
            Err(ReadlineError::Eof) => {
                // ^D
                Some(Command::Back)
            }
            Err(err) => {
                println!("Error: {:?}", err);
                Some(Command::Interrupt)
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

pub fn run() -> Result<()> {
    print_banner();

    // wait("checking tor status");

    // println!("\x1b[1m[\x1b[32m*\x1b[0;1m]\x1b[0m updating registry...");
    // wait("updating registry");

    let _db = worker::spawn_fn("Connecting to database", || {
        let db = SqliteConnection::establish("foo.db")
            .context("Failed to connect to database")?;
        migrations::run(&db)
            .context("Failed to run migrations")?;
        Ok(db)
    }, false)?;

    let mut rl = Readline::new();

    loop {
        match rl.next() {
            Some(Command::Add) => println!("add"),
            Some(Command::Back) => if rl.take_module().is_none() {
                break;
            },
            Some(Command::Run) => {
                if let Some(module) = rl.module() {
                    worker::spawn(module);
                } else {
                    eprintln!("Error: no module selected");
                }
            },
            // TODO: show global settings
            // TODO: if module is some, show module settings
            // TODO: set jobs 25
            Some(Command::Set) => println!("set"),
            Some(Command::Show) => println!("show"),
            Some(Command::Update) => {
                // TODO
                worker::spawn("Updating public suffix list");
                worker::spawn("Updating modules");
            },
            Some(Command::Use(mut args)) => {
                if !args.is_empty() {
                    rl.set_module(args.remove(0));
                } else {
                    eprintln!("Error: argument required");
                }
            },
            Some(Command::Interrupt) => break,
            None => (),
        }
    }

    Ok(())
}
