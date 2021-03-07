pub use failure::{Error, ResultExt, bail, format_err};
pub type Result<T> = ::std::result::Result<T, Error>;
pub use log::{debug, info};

pub use rocket_failure::errors::*;
