use rustyline;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use shellwords;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::str::FromStr;
use shell::Command;


#[derive(Debug, Default)]
pub struct CmdCompleter {
    pub modules: Vec<String>,
}

impl CmdCompleter {
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
