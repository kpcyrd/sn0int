use atty::{self, Stream};
use crate::db;
use crate::engine::Module;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
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

pub struct TermSettings {
    // colors: bool,
    indicate_progress: bool,
}

impl TermSettings {
    pub fn from_env() -> TermSettings {
        if atty::is(Stream::Stdout) {
            TermSettings {
                indicate_progress: true,
            }
        } else {
            TermSettings {
                indicate_progress: false,
            }
        }
    }
}

lazy_static! {
    pub static ref TERM_SETTINGS: TermSettings = TermSettings::from_env();
}

pub trait SpinLogger {
    fn log(&mut self, line: &str);

    fn debug(&mut self, line: &str);

    fn error(&mut self, line: &str);

    fn warn(&mut self, line: &str);

    fn warn_once(&mut self, line: &str);

    fn status(&mut self, status: String);
}

pub struct Spinner {
    indicator: &'static [&'static str],
    status: String,
    i: usize,
    dummy: bool,
    warnings: HashSet<String>,
}

impl Spinner {
    pub fn new(indicator: &'static [&'static str], status: String) -> Spinner {
        let dummy = !TERM_SETTINGS.indicate_progress;
        Spinner {
            indicator,
            status,
            i: 0,
            dummy,
            warnings: HashSet::new(),
        }
    }

    pub fn random(status: String) -> Spinner {
        let indicator = SPINNERS.choose(&mut thread_rng()).unwrap();
        Spinner::new(indicator, status)
    }

    pub fn tick(&mut self) {
        if self.dummy { return; }
        print!("{}", self.tick_bytes());
        io::stdout().flush().unwrap();
    }

    fn tick_bytes(&mut self) -> String {
        if self.i >= self.indicator.len() {
            self.i = 0;
        }

        let s = format!("\r\x1b[2K\x1b[1m[\x1b[32m{}\x1b[0;1m]\x1b[0m {}...", self.indicator[self.i], self.status);
        self.i += 1;

        s
    }

    pub fn done(&self) {
        if self.dummy { return; }
        println!("\r\x1b[2K\x1b[1m[\x1b[32m{}\x1b[0;1m]\x1b[0m {}", '+', self.status);
        io::stdout().flush().unwrap();
    }

    pub fn finish(&mut self, msg: String) {
        self.status(msg);
        self.done();
    }

    #[inline]
    pub fn clear(&self) {
        if self.dummy { return; }
        print!("\r\x1b[2K");
        io::stdout().flush().unwrap();
    }

    pub fn fail(&mut self, err: &str) {
        self.error(err);
        self.clear();
    }
}

impl SpinLogger for Spinner {
    fn log(&mut self, line: &str) {
        if self.dummy { return; }
        println!("\r\x1b[2K\x1b[1m[\x1b[34m{}\x1b[0;1m]\x1b[0m {}", '*', line);
    }

    fn debug(&mut self, line: &str) {
        if self.dummy { return; }
        println!("\r\x1b[2K\x1b[1m[\x1b[34m{}\x1b[0;1m]\x1b[0m {}", '#', line);
    }

    fn error(&mut self, line: &str) {
        if self.dummy { return; }
        println!("\r\x1b[2K\x1b[1m[\x1b[31m{}\x1b[0;1m]\x1b[0m {}", '-', line);
    }

    fn warn(&mut self, line: &str) {
        if self.dummy { return; }
        println!("\r\x1b[2K\x1b[1m[\x1b[33m{}\x1b[0;1m]\x1b[0m {}", '!', line);
    }

    fn warn_once(&mut self, line: &str) {
        if !self.warnings.contains(line) {
            self.warnings.insert(line.into());
            self.warn(line);
        }
    }

    #[inline]
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
    #[inline]
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
    dummy: bool,
    warnings: HashSet<String>,
}

impl StackedSpinners {
    #[inline]
    pub fn new() -> StackedSpinners {
        let dummy = !TERM_SETTINGS.indicate_progress;
        StackedSpinners {
            spinners: HashMap::new(),
            drawn: 0,
            dummy,
            warnings: HashSet::new(),
        }
    }

    pub fn add(&mut self, key: String, status: String) {
        let s = Spinner::random(status);
        self.spinners.insert(key, s);
    }

    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<Spinner> {
        self.spinners.remove(key)
    }

    #[inline]
    pub fn jump2start(&mut self) {
        if self.drawn > 0 {
            print!("\r\x1b[2K\x1b[{}A", self.drawn);
            io::stdout().flush().unwrap();
            self.drawn = 0;
        }
    }

    pub fn tick(&mut self) {
        if self.dummy { return; }
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

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.spinners.is_empty()
    }

    #[inline]
    pub fn clear(&self) {
        if self.dummy { return; }
        print!("\r\x1b[2K");
        io::stdout().flush().unwrap();
    }

    #[inline]
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

    fn warn(&mut self, line: &str) {
        self.jump2start();
        println!("\r\x1b[2K\x1b[1m[\x1b[33m{}\x1b[0;1m]\x1b[0m {}", '!', line);
    }

    #[inline]
    fn warn_once(&mut self, line: &str) {
        if !self.warnings.contains(line) {
            self.warnings.insert(line.into());
            self.warn(line);
        }
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
    #[inline]
    pub fn new<I: Into<String>>(s: &'a mut T, prefix: I) -> PrefixedLogger<T> {
        PrefixedLogger {
            s,
            prefix: prefix.into(),
        }
    }
}

impl<'a, T: SpinLogger> SpinLogger for PrefixedLogger<'a, T> {
    #[inline]
    fn log(&mut self, line: &str) {
        self.s.log(&format!("{:50}: {}", self.prefix, line))
    }

    #[inline]
    fn debug(&mut self, line: &str) {
        self.s.debug(&format!("{:50}: {}", self.prefix, line))
    }

    #[inline]
    fn error(&mut self, line: &str) {
        self.s.error(&format!("{:50}: {}", self.prefix, line))
    }

    #[inline]
    fn warn(&mut self, line: &str) {
        self.s.warn(&format!("{:50}: {}", self.prefix, line))
    }

    #[inline]
    fn warn_once(&mut self, line: &str) {
        self.s.warn_once(line)
    }

    #[inline]
    fn status(&mut self, status: String) {
        self.s.status(format!("{:50}: {}", self.prefix, status))
    }
}
