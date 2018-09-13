use errors::*;

use diesel::prelude::*;
use dirs;
use migrations;
use std::fs;
use std::ops::Deref;
use worker;


pub struct Database {
    name: String,
    db: SqliteConnection,
}

impl Deref for Database {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl Database {
    pub fn establish<I: Into<String>>(name: I) -> Result<Database> {
        // TODO: enforce safe name for database
        let name = name.into();

        // create parent folder
        let path = dirs::data_dir().ok_or_else(|| format_err!("Failed to find data directory"))?;
        let path = path.join("sn0int");
        fs::create_dir_all(&path)
            .context("Failed to create data directory")?;

        let path = path.join(name.clone() + ".db");
        let path = path.into_os_string().into_string()
            .map_err(|_| format_err!("Failed to convert db path to utf-8"))?;

        let db = worker::spawn_fn("Connecting to database", || {
            let db = SqliteConnection::establish(&path)
                .context("Failed to connect to database")?;
            migrations::run(&db)
                .context("Failed to run migrations")?;
            Ok(db)
        }, false)?;

        Ok(Database {
            name,
            db,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn db(&self) -> &SqliteConnection {
        &self.db
    }
}
