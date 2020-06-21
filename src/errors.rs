pub use failure::{bail, format_err, Error, ResultExt};
pub use log::{debug, error, info, trace, warn};
pub type Result<T> = ::std::result::Result<T, Error>;
