use crate::errors::*;

use std::io::{self, Write};
use std::str::FromStr;


pub fn read_line() -> Result<String> {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let buf = buf.trim().to_string();
    Ok(buf)
}

pub fn question(text: &str) -> Result<String> {
    print!("\x1b[1m[\x1b[34m?\x1b[0;1m]\x1b[0m {}: ", text);
    io::stdout().flush()?;

    read_line()
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
