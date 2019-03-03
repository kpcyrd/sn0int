use crate::errors::*;
use crate::db;


#[get("/health")]
pub fn health(_connection: db::Connection) -> ApiResult<()> {
    Ok(())
}
