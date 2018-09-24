use diesel::pg::PgConnection;
use diesel::r2d2;

type ManagedPgConn = r2d2::ConnectionManager<PgConnection>;
type Pool = r2d2::Pool<ManagedPgConn>;


pub fn init(database_url: &str) -> Pool {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::new(manager).expect("Failed to create pool.")
}
