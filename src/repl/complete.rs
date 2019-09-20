use rustyline::Context;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use std::borrow::Cow;


pub struct ReplCompleter;

impl rustyline::Helper for ReplCompleter {}

impl Completer for ReplCompleter {
    type Candidate = String;

    #[inline]
    fn complete(&self, _line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<String>)> {
        if pos == 0 {
            Ok((0, vec![
                String::from("return "),
            ]))
        } else {
            Ok((0, vec![]))
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
    #[inline]
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}
