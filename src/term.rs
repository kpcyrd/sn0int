use crate::db;
use crate::engine::Module;
use rand::prelude::*;
use std::collections::HashMap;
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

pub trait SpinLogger {
    fn log(&mut self, line: &str);

    fn debug(&mut self, line: &str);

    fn error(&mut self, line: &str);

    fn status(&mut self, status: String);
}

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

    pub fn random(status: String) -> Spinner {
        let indicator = SPINNERS.choose(&mut thread_rng()).unwrap();
        Spinner::new(indicator, status)
    }

    pub fn tick(&mut self) {
        print!("{}", self.tick_bytes());
        io::stdout().flush().unwrap();
    }

    pub fn tick_bytes(&mut self) -> String {
        if self.i >= self.indicator.len() {
            self.i = 0;
        }

        let s = format!("\r\x1b[2K\x1b[1m[\x1b[32m{}\x1b[0;1m]\x1b[0m {}...", self.indicator[self.i], self.status);
        self.i += 1;

        s
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

    pub fn fail(&mut self, err: &str) {
        self.error(err);
        self.clear();
    }
}

impl SpinLogger for Spinner {
    fn log(&mut self, line: &str) {
        println!("\r\x1b[2K\x1b[1m[\x1b[34m{}\x1b[0;1m]\x1b[0m {}", '*', line);
    }

    fn debug(&mut self, line: &str) {
        println!("\r\x1b[2K\x1b[1m[\x1b[34m{}\x1b[0;1m]\x1b[0m {}", '#', line);
    }

    fn error(&mut self, line: &str) {
        println!("\r\x1b[2K\x1b[1m[\x1b[31m{}\x1b[0;1m]\x1b[0m {}", '-', line);
    }

    fn status(&mut self, status: String) {
        self.status = status;
    }
}

pub fn success(line: &str) {
    println!("\x1b[1m[\x1b[34m{}\x1b[0;1m]\x1b[0m {}", '*', line);
}

pub fn info(line: &str) {
    println!("\x1b[1m[\x1b[32m{}\x1b[0;1m]\x1b[0m {}", '+', line);
}

pub fn debug(line: &str) {
    println!("\x1b[2K\x1b[1m[\x1b[34m{}\x1b[0;1m]\x1b[0m {}", '#', line);
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
    // TODO: wrapper type that holds module+options
    pub target: Option<db::Filter>,
}

impl Prompt {
    pub fn new(workspace: String) -> Prompt {
        Prompt {
            workspace,
            module: None,
            target: None,
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

pub struct StackedSpinners {
    spinners: HashMap<String, Spinner>,
    drawn: usize,
}

impl StackedSpinners {
    pub fn new() -> StackedSpinners {
        StackedSpinners {
            spinners: HashMap::new(),
            drawn: 0,
        }
    }

    pub fn add(&mut self, key: String, status: String) {
        let s = Spinner::random(status);
        self.spinners.insert(key, s);
    }

    pub fn remove(&mut self, key: &String) -> Option<Spinner> {
        self.spinners.remove(key)
    }

    pub fn jump2start(&mut self) {
        if self.drawn > 0 {
            print!("\r\x1b[2K\x1b[{}A", self.drawn);
            self.drawn = 0;
        }
    }

    pub fn tick(&mut self) {
        self.jump2start();

        if self.spinners.is_empty() {
            return;
        }

        let n = self.spinners.len() -1;
        for (i, (_, s)) in self.spinners.iter_mut().enumerate() {
            print!("{}", s.tick_bytes());
            if i < n {
                print!("\n");
                self.drawn += 1;
            }
        }
        io::stdout().flush().unwrap();
    }

    pub fn is_empty(&self) -> bool {
        self.spinners.is_empty()
    }

    pub fn clear(&self) {
        print!("\r\x1b[2K");
    }

    pub fn prefixed<I: Into<String>>(&mut self, name: I) -> PrefixedLogger<StackedSpinners> {
        PrefixedLogger::new(self, name)
    }
}

impl SpinLogger for StackedSpinners {
    fn log(&mut self, line: &str) {
        self.jump2start();
        println!("\r\x1b[2K\x1b[1m[\x1b[34m{}\x1b[0;1m]\x1b[0m {}", '*', line);
    }

    fn debug(&mut self, line: &str) {
        self.jump2start();
        println!("\r\x1b[2K\x1b[1m[\x1b[34m{}\x1b[0;1m]\x1b[0m {}", '#', line);
    }

    fn error(&mut self, line: &str) {
        self.jump2start();
        println!("\r\x1b[2K\x1b[1m[\x1b[31m{}\x1b[0;1m]\x1b[0m {}", '-', line);
    }

    fn status(&mut self, status: String) {
        self.error(&format!("TODO: set status: {:?}", status));
    }
}

pub struct PrefixedLogger<'a, T: 'a + SpinLogger> {
    s: &'a mut T,
    prefix: String,
}

impl<'a, T: SpinLogger> PrefixedLogger<'a, T> {
    pub fn new<I: Into<String>>(s: &'a mut T, prefix: I) -> PrefixedLogger<T> {
        PrefixedLogger {
            s,
            prefix: prefix.into(),
        }
    }
}

impl<'a, T: SpinLogger> SpinLogger for PrefixedLogger<'a, T> {
    fn log(&mut self, line: &str) {
        self.s.log(&format!("{:50}: {}", self.prefix, line))
    }

    fn debug(&mut self, line: &str) {
        self.s.debug(&format!("{:50}: {}", self.prefix, line))
    }

    fn error(&mut self, line: &str) {
        self.s.error(&format!("{:50}: {}", self.prefix, line))
    }

    fn status(&mut self, status: String) {
        self.s.status(format!("{:50}: {}", self.prefix, status))
    }
}
