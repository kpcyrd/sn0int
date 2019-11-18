use sn0int_registry::errors::*;
use sn0int_registry::db;


#[get("/health")]
pub fn health(_connection: db::Connection) -> ApiResult<()> {
    Ok(())
}
