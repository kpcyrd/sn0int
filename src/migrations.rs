#![allow(unused_imports)]
use crate::errors::*;

use diesel::sqlite::*;

embed_migrations!();

pub fn run(conn: &SqliteConnection) -> Result<()> {
    embedded_migrations::run(conn)?;
    Ok(())
}
