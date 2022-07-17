use crate::errors::*;
use std::path::Path;
use rustyline::{self, CompletionType, EditMode, Editor};
pub use rustyline::error::ReadlineError;


pub struct Readline<T: rustyline::Helper> {
    rl: Editor<T>,
}

impl Readline<()> {
    #[inline]
    pub fn new() -> Result<Readline<()>> {
        Readline::init(None)
    }
}

impl<T: rustyline::Helper> Readline<T> {
    #[inline]
    pub fn with(helper: T) -> Result<Readline<T>> {
        Readline::init(Some(helper))
    }

    fn init(helper: Option<T>) -> Result<Readline<T>> {
        let rl_config = rustyline::Config::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();

        let mut rl: Editor<T> = Editor::with_config(rl_config)?;
        rl.set_helper(helper);

        Ok(Readline {
            rl,
        })
    }

    #[inline]
    pub fn save_history<P: AsRef<Path>>(&mut self, path: &P) -> Result<()> {
        self.rl.save_history(path)
            .map_err(Error::from)
    }

    #[inline]
    pub fn load_history<P: AsRef<Path>>(&mut self, path: &P) -> Result<()> {
        self.rl.load_history(path)
            .map_err(Error::from)
    }

    #[inline]
    pub fn add_history_entry<S: AsRef<str> + Into<String>>(&mut self, line: S) {
        self.rl.add_history_entry(line);
    }

    #[inline]
    pub fn readline(&mut self, prompt: &str) -> rustyline::Result<String> {
        self.rl.readline(prompt)
    }

    #[inline]
    pub fn helper_mut(&mut self) -> Option<&mut T> {
        self.rl.helper_mut()
    }
}
