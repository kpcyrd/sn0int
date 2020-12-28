use crate::repl::tokenize::{self, Token};
use rustyline::Context;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use std::borrow::Cow;


#[derive(Default)]
pub struct ReplCompleter {
    globals: Vec<String>,
}

impl ReplCompleter {
    pub fn set(&mut self, globals: Vec<String>) {
        self.globals = globals;
    }
}

impl rustyline::Helper for ReplCompleter {}
impl rustyline::validate::Validator for ReplCompleter {}

impl Completer for ReplCompleter {
    type Candidate = String;

    #[inline]
    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<String>)> {
        if pos == 0 {
            Ok((0, vec![
                String::from("return "),
            ]))
        } else {
            let filter = match tokenize::parse_last(&line[..pos]) {
                Token::Name(name) => name,
                Token::Empty => String::new(),
                _ => return Ok((0, vec![])),
            };

            let mut options = Vec::new();
            for g in &self.globals {
                if g.starts_with(&filter) {
                    options.push(g.to_string());
                }
            }
            Ok((pos - filter.len(), options))
        }
    }
}

impl Highlighter for ReplCompleter {
    #[inline]
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }
}

impl Hinter for ReplCompleter {
    type Hint = String;

    #[inline]
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}
