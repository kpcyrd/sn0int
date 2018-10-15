use errors::*;
use db::{Database, Filter};
use std::fmt;
use schema::*;


#[derive(Debug, Serialize, Deserialize)]
pub enum Object {
    Domain(NewDomainOwned),
    Subdomain(NewSubdomainOwned),
    IpAddr(NewIpAddrOwned),
    SubdomainIpAddr(NewSubdomainIpAddr),
    Url(NewUrlOwned),
    Email(NewEmailOwned),
}

impl Object {
    pub fn printable(&self, db: &Database) -> Result<String> {
        Ok(match self {
            Object::Domain(x) => format!("Domain: {}", x.printable(db)?),
            Object::Subdomain(x) => format!("Subdomain: {}", x.printable(db)?),
            Object::IpAddr(x) => format!("IpAddr: {}", x.printable(db)?),
            Object::SubdomainIpAddr(x) => x.printable(db)?.to_string(),
            Object::Url(x) => format!("Url: {}", x.printable(db)?),
            Object::Email(x) => format!("Email: {}", x.printable(db)?),
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

pub trait Scopable: Model {
    fn scoped(&self) -> bool;

    fn scope(db: &Database, filter: &Filter) -> Result<usize>;

    fn noscope(db: &Database, filter: &Filter) -> Result<usize>;
}

pub trait Printable<T: Sized> {
    fn printable(&self, db: &Database) -> Result<T>;
}

pub trait Detailed: Scopable {
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

mod email;
pub use self::email::*;
