use engine::Module;
use rand::prelude::*;
use std::fmt;
use std::io;
use std::io::prelude::*;

// https://github.com/Gallopsled/pwntools/blob/dev/pwnlib/term/spinners.py
static SPINNERS: &[&[&str]] = &[
    &["/.......","./......","../.....",".../....","..../...","...../..","....../.",
     ".......\\","......\\.",".....\\..","....\\...","...\\....","..\\.....",".\\......"],
    &["|", "/", "-", "\\"],
    &["q", "p", "b", "d"],
    &[".", "o", "O", "0", "*", " ", " ", " "],
    &["▁", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃"],
    &["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"],
    // &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
    // &["◢", "◢", "◣", "◣", "◤", "◤", "◥", "◥"],
    &["◐", "◓", "◑", "◒"],
    &["▖", "▘", "▝", "▗"],
    &[".", "o", "O", "°", " ", " ", "°", "O", "o", ".", " ", " "],
    // &["<", "<", "∧", "∧", ">", ">", "v", "v"],
];

pub struct Spinner {
    indicator: &'static [&'static str],
    task: String,
    i: usize,
}

impl Spinner {
    pub fn new(indicator: &'static [&'static str], task: String) -> Spinner {
        Spinner {
            indicator,
            task,
            i: 0,
        }
    }

    pub fn status(&mut self, task: String) {
        self.task = task;
    }

    pub fn random(task: String) -> Spinner {
        let indicator = thread_rng().choose(SPINNERS).unwrap();
        Spinner::new(indicator, task)
    }

    pub fn tick(&mut self) {
        if self.i >= self.indicator.len() {
            self.i = 0;
        }

        print!("\r\x1b[1m[\x1b[32m{}\x1b[0;1m]\x1b[0m {}...", self.indicator[self.i], self.task);
        io::stdout().flush().unwrap();

        self.i += 1;
    }

    pub fn log(&self, line: &str) {
        println!("\r\x1b[2K\x1b[1m[\x1b[34m{}\x1b[0;1m]\x1b[0m {}", '*', line);
    }

    pub fn error(&self, line: &str) {
        println!("\r\x1b[2K\x1b[1m[\x1b[31m{}\x1b[0;1m]\x1b[0m {}", '-', line);
    }

    pub fn done(&self) {
        println!("\r\x1b[2K\x1b[1m[\x1b[32m{}\x1b[0;1m]\x1b[0m {}...", '+', self.task);
    }

    pub fn finish(&mut self, task: String) {
        self.status(task);
        self.done();
    }

    pub fn clear(&self) {
        print!("\r\x1b[2K");
    }
}

pub fn success(line: &str) {
    println!("\x1b[1m[\x1b[34m{}\x1b[0;1m]\x1b[0m {}", '*', line);
}

pub fn info(line: &str) {
    println!("\x1b[1m[\x1b[32m{}\x1b[0;1m]\x1b[0m {}", '+', line);
}

pub fn warn(line: &str) {
    eprintln!("\x1b[1m[\x1b[33m{}\x1b[0;1m]\x1b[0m {}", '!', line);
}

pub fn error(line: &str) {
    eprintln!("\x1b[1m[\x1b[31m{}\x1b[0;1m]\x1b[0m {}", '-', line);
}

pub struct Prompt {
    pub workspace: String,
    pub module: Option<Module>,
}

impl Prompt {
    pub fn new(workspace: String) -> Prompt {
        Prompt {
            workspace,
            module: None,
        }
    }
}

impl fmt::Display for Prompt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[sn0int][{}]", self.workspace)?;
        if let Some(module) = &self.module {
            write!(f, "[{}]", module.canonical())?;
        }
        write!(f, " > ")
    }
}
