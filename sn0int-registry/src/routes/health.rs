use sn0int_registry::db;
use sn0int_registry::errors::*;

#[get("/health")]
pub fn health(_connection: db::Connection) -> ApiResult<()> {
    Ok(())
}
