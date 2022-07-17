use crate::errors::*;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use rustyline::error::ReadlineError;
use std::iter;
use std::str::FromStr;

pub fn random_string(len: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(len)
        .collect()
}

pub fn read_line(prompt: &str) -> Result<String> {
    let mut rl = rustyline::Editor::<()>::new()?;
    let mut line = rl.readline(prompt)
        .map_err(|err| match err {
            ReadlineError::Eof => format_err!("Failed to read line from input"),
            ReadlineError::Interrupted => format_err!("Prompt has been canceled"),
            err => err.into(),
        })?;
    if let Some(idx) = line.find('\n') {
        line.truncate(idx);
    }
    info!("Read from prompt: {:?}", line);
    Ok(line)
}

pub fn question(text: &str) -> Result<String> {
    let prompt = format!("\x1b[1m[\x1b[34m?\x1b[0;1m]\x1b[0m {}: ", text);
    read_line(&prompt)
}

pub fn question_opt(text: &str) -> Result<Option<String>> {
    let answer = question(text)?;

    if !answer.is_empty() {
        Ok(Some(answer))
    } else {
        Ok(None)
    }
}

pub fn question_typed_opt<T: FromStr>(text: &str) -> Result<Option<T>> {
    let answer = question(text)?;

    if !answer.is_empty() {
        let answer = answer.parse()
            .map_err(|_| format_err!("Failed to parse input"))?;
        Ok(Some(answer))
    } else {
        Ok(None)
    }
}

pub fn question_or<I: Into<String>>(text: &str, default: I) -> Result<String> {
    let default = default.into();
    let answer = question(&format!("{} [{}]", text, default))?;

    if !answer.is_empty() {
        Ok(answer)
    } else {
        Ok(default)
    }
}

#[inline]
fn yes_no(text: &str, default: bool) -> Result<bool> {
    let answer = question(text)?;
    let answer = answer.to_lowercase();

    Ok(match answer.as_str() {
        "" => default,
        "y" => true,
        "n" => false,
        _ => bail!("invalid input"),
    })
}

pub fn yes_else_no(text: &str) -> Result<bool> {
    yes_no(&format!("{} [Y/n]", text), true)
}

pub fn no_else_yes(text: &str) -> Result<bool> {
    yes_no(&format!("{} [y/N]", text), false)
}
