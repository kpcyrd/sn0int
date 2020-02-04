pub use log::{trace, debug, info, warn, error};
pub use failure::{Error, ResultExt, format_err, bail};
pub type Result<T> = ::std::result::Result<T, Error>;
