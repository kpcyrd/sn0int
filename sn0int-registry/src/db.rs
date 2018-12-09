use crate::errors::*;
use std::io;
use std::thread;
use std::time::Duration;
use std::ops::Deref;
use diesel::Connection as ConnectionTrait;
use diesel::pg::PgConnection;
use diesel::r2d2;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};


type ManagedPgConn = r2d2::ConnectionManager<PgConnection>;
type Pool = r2d2::Pool<ManagedPgConn>;

pub fn init(database_url: &str) -> Pool {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::new(manager).expect("Failed to create pool.")
}

/// Connection request guard type: a wrapper around an r2d2 pooled connection.
pub struct Connection(pub r2d2::PooledConnection<ManagedPgConn>);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for Connection {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Connection(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

// For the convenience of using an &Connection as an &MysqlConnection.
impl Deref for Connection {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

embed_migrations!("migrations");

pub fn wait_for_db(url: &str, attempts: u8) -> Result<PgConnection> {
    for _ in 0..attempts {
        match PgConnection::establish(url) {
            Ok(connection) => return Ok(connection),
            Err(err) => eprintln!("Waiting for db: {}", err),
        }
        thread::sleep(Duration::from_secs(3));
    }

    bail!("Database didn't come online in time")
}

pub fn setup_db(url: &str, attempts: u8) -> Result<()> {
    let connection = wait_for_db(url, attempts)?;
    embedded_migrations::run_with_output(&connection, &mut io::stdout())?;
    Ok(())
}
