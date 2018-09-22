use errors::*;
use db::{Database, Filter};
use std::fmt;
use schema::*;


#[derive(Debug, Serialize, Deserialize)]
pub enum Object {
    Subdomain(NewSubdomainOwned),
    IpAddr(NewIpAddrOwned),
    SubdomainIpAddr(NewSubdomainIpAddr),
    Url(NewUrlOwned),
}

impl Object {
    pub fn printable(&self, db: &Database) -> Result<String> {
        Ok(match self {
            Object::Subdomain(x) => format!("Subdomain: {}", x.printable(db)?),
            Object::IpAddr(x) => format!("IpAddr: {}", x.printable(db)?),
            Object::SubdomainIpAddr(x) => x.printable(db)?.to_string(),
            Object::Url(x) => format!("Url: {}", x.printable(db)?),
        })
    }
}

pub trait Model: Sized {
    type ID: ?Sized;

    fn list(db: &Database) -> Result<Vec<Self>>;

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>>;

    fn by_id(db: &Database, id: i32) -> Result<Self>;

    fn id(db: &Database, query: &Self::ID) -> Result<i32>;

    fn id_opt(db: &Database, query: &Self::ID) -> Result<Option<i32>>;
}

pub trait Printable<T: Sized> {
    fn printable(&self, db: &Database) -> Result<T>;
}

pub trait Detailed {
    type T: fmt::Display;

    fn detailed(&self, db: &Database) -> Result<Self::T>;
}

mod domain;
pub use self::domain::*;

mod subdomain;
pub use self::subdomain::*;

mod ipaddr;
pub use self::ipaddr::*;

mod subdomain_ipaddr;
pub use self::subdomain_ipaddr::*;

mod url;
pub use self::url::*;
