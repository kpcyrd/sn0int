use engine::Module;
use rand::prelude::*;
use std::fmt;
use std::io;
use std::io::prelude::*;

// https://github.com/Gallopsled/pwntools/blob/dev/pwnlib/term/spinners.py
// https://github.com/gernest/wow/blob/master/spin/spinners.go
pub static SPINNERS: &[&[&str]] = &[
    &["/.......","./......","../.....",".../....","..../...","...../..","....../.",
     ".......\\","......\\.",".....\\..","....\\...","...\\....","..\\.....",".\\......"],
    &["|", "/", "-", "\\"],
    &["q", "p", "b", "d"],
    &[".", "o", "O", "0", "*", " ", " ", " "],
    &["▁", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃"],
    &["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"],
    &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
    &["◢", "◣", "◤", "◥"],
    &["◐", "◓", "◑", "◒"],
    &["▖", "▘", "▝", "▗"],
    &[".", "o", "O", "°", " ", " ", "°", "O", "o", ".", " ", " "],
    &["<", "∧", ">", "v"],
    &["◜", "◠", "◝", "◞", "◡", "◟"],
    &["▹▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▹▸▹", "▹▹▹▹▸"],
    &["    ", "=   ", "==  ", "=== ", " ===", "  ==", "   =", "    ",
      "   =", "  ==", " ===", "====", "=== ", "==  ", "=   "],
    &["▌", "▀", "▐", "▄"],
    &["◴", "◴", "◷", "◷", "◶", "◶", "◵", "◵"],
    &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
    &["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"],
    &["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"],
    &["⢀⠀", "⡀⠀", "⠄⠀", "⢂⠀", "⡂⠀", "⠅⠀", "⢃⠀", "⡃⠀", "⠍⠀", "⢋⠀", "⡋⠀", "⠍⠁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⢈⠩", "⡀⢙", "⠄⡙", "⢂⠩", "⡂⢘", "⠅⡘", "⢃⠨", "⡃⢐", "⠍⡐", "⢋⠠", "⡋⢀", "⠍⡁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⠈⠩", "⠀⢙", "⠀⡙", "⠀⠩", "⠀⢘", "⠀⡘", "⠀⠨", "⠀⢐", "⠀⡐", "⠀⠠", "⠀⢀", "⠀⡀"],
    &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
    &["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"],
    &["⠄", "⠆", "⠇", "⠋", "⠙", "⠸", "⠰", "⠠", "⠰", "⠸", "⠙", "⠋", "⠇", "⠆"],
    &["⠁", "⠉", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠤", "⠄", "⠄", "⠤", "⠴", "⠲", "⠒", "⠂", "⠂", "⠒", "⠚", "⠙", "⠉", "⠁"],
    &["⢹", "⢺", "⢼", "⣸", "⣇", "⡧", "⡗", "⡏"],
    &["▙", "▛", "▜", "▟"],
    &["☱", "☲", "☴"],
    &["▓", "▒", "░", "▒"],
    &["⠂       ", "⠈       ", " ⠂      ", " ⠠      ", "  ⡀     ", "  ⠠     ", "   ⠂    ", "   ⠈    ", "    ⠂   ", "    ⠠   ", "     ⡀  ", "     ⠠  ", "      ⠂ ", "      ⠈ ", "       ⠂", "       ⠠", "       ⡀", "      ⠠ ", "      ⠂ ", "     ⠈  ", "     ⠂  ", "    ⠠   ", "    ⡀   ", "   ⠠    ", "   ⠂    ", "  ⠈     ", "  ⠂     ", " ⠠      ", " ⡀      ", "⠠       "],
    &["|\\____________", "_|\\___________", "__|\\__________", "___|\\_________", "____|\\________", "_____|\\_______", "______|\\______", "_______|\\_____", "________|\\____", "_________|\\___", "__________|\\__", "___________|\\_", "____________|\\", "____________/|", "___________/|_", "__________/|__", "_________/|___", "________/|____", "_______/|_____", "______/|______", "_____/|_______", "____/|________", "___/|_________", "__/|__________", "_/|___________", "/|____________"],
    &[".  ", ".. ", "...", " ..", "  .", "   "],
    &["    ", ".   ", "..  ", "... ", " ...", "  ..", "   .", "    ",
      "   .", "  ..", " ...", "....", "... ", "..  ", ".   "],
    &[" ", "▘", "▀", "▜", "█", "▟", "▄", "▖"],
    &["⠃", "⠊", "⠒", "⠢", "⠆", "⠰", "⠔", "⠒", "⠑", "⠘"],
    &[" ", "⠁", "⠉", "⠙", "⠚", "⠖", "⠦", "⠤", "⠠"],
];

pub struct Spinner {
    indicator: &'static [&'static str],
    status: String,
    i: usize,
}

impl Spinner {
    pub fn new(indicator: &'static [&'static str], status: String) -> Spinner {
        Spinner {
            indicator,
            status,
            i: 0,
        }
    }

    pub fn status(&mut self, status: String) {
        self.status = status;
    }

    pub fn random(status: String) -> Spinner {
        let indicator = thread_rng().choose(SPINNERS).unwrap();
        Spinner::new(indicator, status)
    }

    pub fn tick(&mut self) {
        if self.i >= self.indicator.len() {
            self.i = 0;
        }

        print!("\r\x1b[1m[\x1b[32m{}\x1b[0;1m]\x1b[0m {}...", self.indicator[self.i], self.status);
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
        println!("\r\x1b[2K\x1b[1m[\x1b[32m{}\x1b[0;1m]\x1b[0m {}", '+', self.status);
    }

    pub fn finish(&mut self, msg: String) {
        self.status(msg);
        self.done();
    }

    pub fn clear(&self) {
        print!("\r\x1b[2K");
    }

    pub fn fail(&self, err: &str) {
        self.error(err);
        self.clear();
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
