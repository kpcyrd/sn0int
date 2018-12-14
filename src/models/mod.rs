use crate::errors::*;
use crate::db::{Database, Filter};
use std::fmt;
use crate::schema::*;


#[derive(Debug, Serialize, Deserialize)]
pub enum Insert {
    Domain(NewDomainOwned),
    Subdomain(NewSubdomainOwned),
    IpAddr(NewIpAddrOwned),
    SubdomainIpAddr(NewSubdomainIpAddr),
    Url(NewUrlOwned),
    Email(NewEmailOwned),
    PhoneNumber(NewPhoneNumberOwned),
}

impl Insert {
    pub fn value(&self) -> &str {
        match self {
            Insert::Domain(x) => &x.value,
            Insert::Subdomain(x) => &x.value,
            Insert::IpAddr(x) => &x.value,
            Insert::SubdomainIpAddr(_x) => unimplemented!("SubdomainIpAddr doesn't have value field"),
            Insert::Url(x) => &x.value,
            Insert::Email(x) => &x.value,
            Insert::PhoneNumber(x) => &x.value,
        }
    }

    pub fn printable(&self, db: &Database) -> Result<String> {
        Ok(match self {
            Insert::Domain(x) => format!("Domain: {}", x.printable(db)?),
            Insert::Subdomain(x) => format!("Subdomain: {}", x.printable(db)?),
            Insert::IpAddr(x) => format!("IpAddr: {}", x.printable(db)?),
            Insert::SubdomainIpAddr(x) => x.printable(db)?.to_string(),
            Insert::Url(x) => format!("Url: {}", x.printable(db)?),
            Insert::Email(x) => format!("Email: {}", x.printable(db)?),
            Insert::PhoneNumber(x) => format!("Email: {}", x.printable(db)?),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Update {
    Subdomain(SubdomainUpdate),
    IpAddr(IpAddrUpdate),
    Url(UrlUpdate),
    Email(EmailUpdate),
    PhoneNumber(PhoneNumberUpdate),
}

impl Update {
    pub fn is_dirty(&self) -> bool {
        match self {
            Update::Subdomain(x) => x.is_dirty(),
            Update::IpAddr(x) => x.is_dirty(),
            Update::Url(x) => x.is_dirty(),
            Update::Email(x) => x.is_dirty(),
            Update::PhoneNumber(x) => x.is_dirty(),
        }
    }
}

impl fmt::Display for Update {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Update::Subdomain(update) => write!(w, "{}", update.to_string()),
            Update::IpAddr(update) => write!(w, "{}", update.to_string()),
            Update::Url(update) => write!(w, "{}", update.to_string()),
            Update::Email(update) => write!(w, "{}", update.to_string()),
            Update::PhoneNumber(update) => write!(w, "{}", update.to_string()),
        }
    }
}

pub trait Model: Sized {
    type ID: ?Sized;

    fn to_string(&self) -> String;

    fn list(db: &Database) -> Result<Vec<Self>>;

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>>;

    fn delete(db: &Database, filter: &Filter) -> Result<usize>;

    fn id(&self) -> i32;

    fn value(&self) -> &Self::ID {
        unimplemented!()
    }

    fn by_id(db: &Database, id: i32) -> Result<Self>;

    fn get_id(db: &Database, query: &Self::ID) -> Result<i32> {
        Self::get(db, query)
            .map(|x| x.id())
    }

    fn get_id_opt(db: &Database, query: &Self::ID) -> Result<Option<i32>> {
        Self::get_opt(db, query)
            .map(|x| x
                .map(|x| x.id()))
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self>;

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>>;
}

pub trait Scopable: Model {
    fn scoped(&self) -> bool;

    fn scope(db: &Database, filter: &Filter) -> Result<usize>;

    fn noscope(db: &Database, filter: &Filter) -> Result<usize>;
}

pub trait InsertableStruct<T: Model>: Upsertable<T> {
    fn value(&self) -> &T::ID;

    fn insert(&self, db: &Database) -> Result<()>;
}

pub trait Upsertable<M> {
    type Update: Upsert;

    #[inline]
    fn upsert_str(insert: Option<&String>, existing: &Option<String>) -> Option<String> {
        if insert != existing.as_ref() { insert.map(|x| x.to_owned()) } else { None }
    }

    #[inline]
    fn upsert_bytes(insert: Option<&Vec<u8>>, existing: &Option<Vec<u8>>) -> Option<Vec<u8>> {
        if insert != existing.as_ref() { insert.map(|x| x.to_owned()) } else { None }
    }

    #[inline]
    fn upsert_opt<T: PartialEq>(insert: Option<T>, existing: &Option<T>) -> Option<T> {
        if insert != *existing { insert } else { None }
    }

    fn upsert(self, existing: &M) -> Self::Update;
}

pub trait Upsert {
    fn is_dirty(&self) -> bool;

    fn generic(self) -> Update;

    fn apply(&self, db: &Database) -> Result<i32>;
}

pub struct NullUpdate {
    pub id: i32,
}

impl Upsert for NullUpdate {
    fn is_dirty(&self) -> bool {
        false
    }

    fn generic(self) -> Update {
        unreachable!("Object doesn't have any immutable fields")
    }

    fn apply(&self, _db: &Database) -> Result<i32> {
        Ok(self.id)
    }
}

pub trait Updateable<M> {
    fn to_string(&self) -> String {
        let mut updates = Vec::new();
        self.fmt(&mut updates);
        updates.join(", ")
    }

    #[inline]
    fn clear_if_equal<T: PartialEq>(update: &mut Option<T>, existing: &Option<T>) {
        if update == existing { update.take(); }
    }

    fn changeset(&mut self, existing: &M);

    #[inline]
    fn push_value<D: fmt::Debug>(updates: &mut Vec<String>, name: &str, value: &Option<D>) {
        if let Some(v) = value {
            updates.push(format!("{} => \x1b[33m{:?}\x1b[0m", name, v));
        }
    }

    #[inline]
    fn push_raw<T: AsRef<str>>(updates: &mut Vec<String>, name: &str, value: Option<T>) {
        if let Some(v) = value {
            updates.push(format!("{} => \x1b[33m{}\x1b[0m", name, v.as_ref()));
        }
    }

    fn fmt(&self, updates: &mut Vec<String>);
}

pub trait Printable<T: Sized> {
    fn printable(&self, db: &Database) -> Result<T>;
}

pub trait Detailed: Scopable {
    type T: fmt::Display;

    fn detailed(&self, db: &Database) -> Result<Self::T>;
}

pub trait DisplayableDetailed {
    fn scoped(&self) -> bool;

    #[inline]
    fn start(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if !self.scoped() {
            write!(w, "\x1b[90m")
        } else {
            Ok(())
        }
    }

    #[inline]
    fn end(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if !self.scoped() {
            write!(w, "\x1b[0m")
        } else {
            Ok(())
        }
    }

    #[inline]
    fn id<D: fmt::Display>(&self, w: &mut fmt::Formatter, v: D) -> fmt::Result {
        if self.scoped() {
            write!(w, "\x1b[32m#{}\x1b[0m, ", v)
        } else {
            write!(w, "#{}, ", v)
        }
    }

    #[inline]
    fn red_display<D: fmt::Display>(&self, w: &mut fmt::Formatter, v: D) -> fmt::Result {
        if self.scoped() {
            write!(w, "\x1b[31m{}\x1b[0m", v)
        } else {
            write!(w, "{}", v)
        }
    }

    #[inline]
    fn green(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if self.scoped() {
            write!(w, "\x1b[32m")
        } else {
            Ok(())
        }
    }

    #[inline]
    fn green_display<D: fmt::Display>(&self, w: &mut fmt::Formatter, v: D) -> fmt::Result {
        if self.scoped() {
            write!(w, "\x1b[32m{}\x1b[0m", v)
        } else {
            write!(w, "{}", v)
        }
    }

    #[inline]
    fn green_debug<D: fmt::Debug>(&self, w: &mut fmt::Formatter, v: D) -> fmt::Result {
        if self.scoped() {
            write!(w, "\x1b[32m{:?}\x1b[0m", v)
        } else {
            write!(w, "{:?}", v)
        }
    }

    #[inline]
    fn yellow(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if self.scoped() {
            write!(w, "\x1b[33m")
        } else {
            Ok(())
        }
    }

    #[inline]
    fn yellow_debug<D: fmt::Debug>(&self, w: &mut fmt::Formatter, v: D) -> fmt::Result {
        if self.scoped() {
            write!(w, "\x1b[33m{:?}\x1b[0m", v)
        } else {
            write!(w, "{:?}", v)
        }
    }

    #[inline]
    fn clear(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if self.scoped() {
            write!(w, "\x1b[0m")
        } else {
            Ok(())
        }
    }

    #[inline]
    fn child<D: fmt::Display>(&self, w: &mut fmt::Formatter, c: D) -> fmt::Result {
        if self.scoped() {
            // if child is unscoped, draw as grey as well
            write!(w, "\n\t\x1b[33m{}\x1b[0m", c)
        } else {
            write!(w, "\n\t\x1b[90m{}\x1b[0m", c)
        }
    }

    fn print(&self, w: &mut fmt::Formatter) -> fmt::Result;

    fn children(&self, w: &mut fmt::Formatter) -> fmt::Result;
}

macro_rules! display_detailed {
    ( $name:ident ) => {
        impl fmt::Display for $name {
            fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
                self.start(w)?;
                self.print(w)?;
                self.end(w)?;
                self.children(w)?;
                Ok(())
            }
        }
    };
}

pub trait LuaInsertToNewOwned {
    type Target;

    fn try_into_new(self) -> Result<Self::Target>;
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

mod phonenumber;
pub use self::phonenumber::*;
