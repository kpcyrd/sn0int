use crate::args::{Args, Completions};
use crate::errors::*;
use rustyline;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use shellwords;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::str::FromStr;
use std::io::stdout;
use structopt::StructOpt;
use crate::shell::Command;
use crate::workspaces;


#[derive(Debug, Default)]
pub struct CmdCompleter {
    pub modules: Vec<String>,
}

impl CmdCompleter {
    pub fn filter(&self, cmd: &str, args: &[String]) -> rustyline::Result<(usize, Vec<String>)> {
        // we can only complete the 2nd argument
        if args.len() != 2 {
            Ok((0, vec![]))
        } else {
            let arg = &args[1];

            let options = &["domains",
                            "subdomains",
                            "ipaddrs",
                            "urls",
                            "emails",
                            "phonenumbers"];

            let results: Vec<String> = options.iter()
                .filter(|x| x.starts_with(arg))
                .map(|x| format!("{} {} ", cmd, x))
                .collect();
            Ok((0, results))
        }
    }
}

impl Completer for CmdCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        if line.len() != pos {
            return Ok((0, vec![]));
        }

        let mut cmd = match shellwords::split(&line) {
            Ok(cmd) => cmd,
            Err(_) => return Ok((0, vec![])),
        };

        // if the line ends with space, try to complete a new arg
        if let Some(last_char) = line.chars().last() {
            if last_char == ' ' {
                cmd.push(String::new());
            }
        }
        let args = cmd.len();

        // we are trying to complete the action
        if args <= 1 {
            let results: Vec<String> = Command::list_all().iter()
                .filter(|x| x.starts_with(line))
                .map(|x| x.to_string() + " ")
                .collect();

            Ok((0, results))
        } else {
            // we can complete arguments for some actions
            let action = match Command::from_str(&cmd[0]) {
                Ok(action) => action,
                Err(_) => return Ok((0, vec![])),
            };

            match action {
                Command::Add => {
                    // we can only complete the 2nd argument
                    if args != 2 {
                        Ok((0, vec![]))
                    } else {
                        let arg = &cmd[1];

                        let options = &["domain",
                                        "subdomain",
                                        "email",
                                        "phonenumber"];

                        let results: Vec<String> = options.iter()
                            .filter(|x| x.starts_with(arg))
                            .map(|x| format!("add {} ", x))
                            .collect();
                        Ok((0, results))
                    }
                },
                Command::Delete => self.filter("delete", &cmd),
                Command::Mod => {
                    // we can only complete the 2nd argument
                    if args != 2 {
                        Ok((0, vec![]))
                    } else {
                        let arg = &cmd[1];

                        let options = &["list",
                                        "install",
                                        "search",
                                        "reload",
                                        "update"];

                        let results: Vec<String> = options.iter()
                            .filter(|x| x.starts_with(arg))
                            .map(|x| format!("mod {} ", x))
                            .collect();
                        Ok((0, results))
                    }

                },
                Command::Noscope => self.filter("noscope", &cmd),
                Command::Use => {
                    // we can only complete the 2nd argument
                    if args != 2 {
                        Ok((0, vec![]))
                    } else {
                        let arg = &cmd[1];

                        let results: Vec<String> = self.modules.iter()
                            .filter(|x| x.starts_with(arg))
                            .map(|x| format!("use {} ", x))
                            .collect();
                        Ok((0, results))
                    }
                },
                Command::Scope => self.filter("scope", &cmd),
                Command::Select => self.filter("select", &cmd),
                Command::Workspace => {
                    // we can only complete the 2nd argument
                    if args != 2 {
                        Ok((0, vec![]))
                    } else {
                        let arg = &cmd[1];

                        let results: Vec<String> = match workspaces::list() {
                            Ok(workspaces) => workspaces.iter()
                                .filter(|x| x.starts_with(arg))
                                .map(|x| format!("workspace {} ", x.as_str()))
                                .collect(),
                            _ => Vec::new(),
                        };

                        Ok((0, results))
                    }
                },
                _ => Ok((0, vec![])),
            }
        }
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

pub fn run_generate(args: &Completions) -> Result<()> {
    Args::clap().gen_completions_to("sn0int", args.shell, &mut stdout());
    Ok(())
}
