use crate::errors::*;

pub enum Lazy<T1, T2> {
    Init(Option<T1>),
    Active(T2),
}

pub trait LazyInit<T> {
    fn initialize(self) -> Result<T>;
}

impl<T1: LazyInit<T2>, T2> Lazy<T1, T2> {
    pub fn get(&mut self) -> Result<&mut T2> {
        match self {
            Lazy::Init(init) => {
                let init = init.take()
                    .ok_or_else(|| format_err!("Previous initialization failed"))?;
                *self = Lazy::Active(init.initialize()?);
                self.get()
            },
            Lazy::Active(active) => Ok(active),
        }
    }
}

impl<T1, T2> From<T1> for Lazy<T1, T2> {
    fn from(x: T1) -> Lazy<T1, T2> {
        Lazy::Init(Some(x))
    }
}
