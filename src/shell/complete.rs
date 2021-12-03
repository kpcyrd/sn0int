use crate::args::{Args, Completions};
use crate::autonoscope::RuleType;
use crate::errors::*;
use rustyline::{self, Context};
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use std::borrow::Cow::{self, Owned};
use std::str::FromStr;
use std::io::stdout;
use structopt::StructOpt;
use crate::shell::Command;
use crate::workspaces;


#[derive(Debug, Default)]
pub struct CmdCompleter {
    pub modules: Vec<String>,
    pub keyring: Vec<String>,
}

impl CmdCompleter {
    pub fn filter(&self, cmd: &str, args: &[String]) -> rustyline::Result<(usize, Vec<String>)> {
        // we can only complete the 2nd argument
        if args.len() != 2 {
            Ok((0, vec![]))
        } else {
            Ok(filter_options(cmd, &[
                "domains",
                "subdomains",
                "ipaddrs",
                "urls",
                "emails",
                "phonenumbers",
                "devices",
                "networks",
                "accounts",
                "breaches",
                "images",
                "ports",
                "netblocks",
                "cryptoaddrs",
            ], &args[1]))
        }
    }
}

fn filter_options(prefix: &str, options: &[&str], arg: &str) -> (usize, Vec<String>) {
    let results: Vec<String> = options.iter()
        .filter(|x| x.starts_with(arg))
        .map(|x| format!("{} {} ", prefix, x))
        .collect();
    (0, results)
}

impl Completer for CmdCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<String>)> {
        if line.len() != pos {
            return Ok((0, vec![]));
        }

        let mut cmd = match shellwords::split(line) {
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
                                        "ipnet",
                                        "ipaddr",
                                        "url",
                                        "email",
                                        "phonenumber",
                                        "device",
                                        "network",
                                        "account",
                                        "breach",
                                        "image",
                                        "netblock",
                                        "port",
                                        "cryptoaddr"];

                        let results: Vec<String> = options.iter()
                            .filter(|x| x.starts_with(arg))
                            .map(|x| format!("add {} ", x))
                            .collect();
                        Ok((0, results))
                    }
                },
                Command::Autonoscope | Command::Autoscope => {
                    match (args, cmd.get(1).map(|x| x.as_str())) {
                        (2, _) => {
                            Ok(filter_options(&cmd[0], &[
                                "add",
                                "delete",
                                "list"
                            ], &cmd[1]))
                        },
                        (3, Some("add")) | (3, Some("delete")) => {
                            let base = &cmd[0];
                            let action = &cmd[1];
                            let arg = &cmd[2];

                            let options = RuleType::list_all();

                            let results: Vec<String> = options.iter()
                                .filter(|x| x.starts_with(arg))
                                .map(|x| format!("{} {} {} ", base, action, x))
                                .collect();
                            Ok((0, results))
                        },
                        _ => {
                            Ok((0, vec![]))
                        },
                    }
                },
                Command::Delete => self.filter("delete", &cmd),
                Command::Keyring => {
                    match (args, cmd.get(1).map(|x| x.as_str())) {
                        (2, _) => {
                            Ok(filter_options("keyring", &[
                                "add",
                                "delete",
                                "get",
                                "list"
                            ], &cmd[1]))
                        },
                        (3, Some("get")) | (3, Some("delete")) => {
                            let base = &cmd[0];
                            let action = &cmd[1];
                            let arg = &cmd[2];

                            let results: Vec<String> = self.keyring.iter()
                                .filter(|x| x.starts_with(arg))
                                .map(|x| format!("{} {} {} ", base, action, x))
                                .collect();
                            Ok((0, results))
                        },
                        _ => {
                            Ok((0, vec![]))
                        },
                    }
                },
                Command::Mod => {
                    // we can only complete the 2nd argument
                    if args != 2 {
                        Ok((0, vec![]))
                    } else {
                        Ok(filter_options("mod", &[
                            "list",
                            "install",
                            "search",
                            "reload",
                            "update",
                            "uninstall",
                        ], &cmd[1]))
                    }
                },
                Command::Pkg => {
                    // we can only complete the 2nd argument
                    let subcommand = cmd.get(1).map(|x| x.as_str());

                    let current = &cmd[args - 1];
                    let prev = &cmd[args - 2];

                    match (args, subcommand) {
                        (_, Some("list")) => {
                            let line = &line[..line.len() - current.len() - 1];

                            match prev.as_str() {
                                "--source" => Ok(filter_options(line, &[
                                    "domains",
                                    "subdomains",
                                    "ipaddrs",
                                    "urls",
                                    "emails",
                                    "phonenumbers",
                                    "devices",
                                    "networks",
                                    "accounts",
                                    "breaches",
                                    "images",
                                    "ports",
                                    "netblocks",
                                    "cryptoaddrs",
                                ], current)),
                                "--stealth" => Ok(filter_options(line, &[
                                    "loud",
                                    "normal",
                                    "passive",
                                    "offline",
                                ], current)),
                                _ => if current.starts_with('-') {
                                    Ok(filter_options(line, &[
                                        "--source",
                                        "--stealth",
                                    ], current))
                                } else {
                                    Ok((0, vec![]))
                                },
                            }
                        },
                        (2, Some(subcommand)) => {
                            Ok(filter_options("pkg", &[
                                "list",
                                "install",
                                "search",
                                "reload",
                                "update",
                                "uninstall",
                                "quickstart",
                            ], subcommand))
                        },
                        (_, _) => Ok((0, vec![])),
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
                Command::Rescope => self.filter("rescope", &cmd),
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
    type Hint = String;

    #[inline]
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        // None
        match self.complete(line, pos, ctx) {
            Ok((_, mut cmds)) => if cmds.len() == 1 {
                // TODO: this fails if we complete a 2nd argument
                let hint = cmds.remove(0);
                Some(hint[pos..].to_string())
            } else {
                None
            },
            Err(_) => None,
        }
    }
}

impl Highlighter for CmdCompleter {
    #[inline]
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[90m".to_owned() + hint + "\x1b[m")
    }
}

impl rustyline::Helper for CmdCompleter {}
impl rustyline::validate::Validator for CmdCompleter {}

pub fn run_generate(args: &Completions) -> Result<()> {
    Args::clap().gen_completions_to("sn0int", args.shell, &mut stdout());
    Ok(())
}
