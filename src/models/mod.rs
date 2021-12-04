use crate::db::{Database, Table, Filter, Family};
use serde::{Serialize, Deserialize};
use crate::engine::ctx::State;
use crate::errors::*;
use crate::fmt;
use crate::schema::*;
use std::borrow::Cow;
use std::sync::Arc;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Insert {
    Domain(NewDomain),
    Subdomain(NewSubdomain),
    IpAddr(NewIpAddr),
    SubdomainIpAddr(NewSubdomainIpAddr),
    Url(NewUrl),
    Email(NewEmail),
    PhoneNumber(NewPhoneNumber),
    Device(NewDevice),
    Network(NewNetwork),
    NetworkDevice(NewNetworkDevice),
    Account(NewAccount),
    Breach(NewBreach),
    BreachEmail(NewBreachEmail),
    Image(NewImage),
    Port(NewPort),
    Netblock(NewNetblock),
    CryptoAddr(NewCryptoAddr),
}

impl Insert {
    pub fn value(&self, db: &Database) -> Result<Cow<String>> {
        let value = match self {
            Insert::Domain(x) => Cow::Borrowed(&x.value),
            Insert::Subdomain(x) => Cow::Borrowed(&x.value),
            Insert::IpAddr(x) => Cow::Borrowed(&x.value),
            Insert::SubdomainIpAddr(x) => {
                let subdomain = Subdomain::by_id(db, x.subdomain_id)?;
                let ipaddr = IpAddr::by_id(db, x.ip_addr_id)?;
                Cow::Owned(format!("{}+{}", subdomain.value, ipaddr.value))
            },
            Insert::Url(x) => Cow::Borrowed(&x.value),
            Insert::Email(x) => Cow::Borrowed(&x.value),
            Insert::PhoneNumber(x) => Cow::Borrowed(&x.value),
            Insert::Device(x) => Cow::Borrowed(&x.value),
            Insert::Network(x) => Cow::Borrowed(&x.value),
            Insert::NetworkDevice(x) => {
                let network = Network::by_id(db, x.network_id)?;
                let device = Device::by_id(db, x.device_id)?;
                Cow::Owned(format!("{}+{}", network.value, device.value))
            },
            Insert::Account(x) => Cow::Borrowed(&x.value),
            Insert::Breach(x) => Cow::Borrowed(&x.value),
            Insert::BreachEmail(x) => {
                let breach = Breach::by_id(db, x.breach_id)?;
                let email = Email::by_id(db, x.email_id)?;
                Cow::Owned(format!("{}+{}", breach.value, email.value))
            }
            Insert::Image(x) => Cow::Borrowed(&x.value),
            Insert::Port(x) => Cow::Borrowed(&x.value),
            Insert::Netblock(x) => Cow::Borrowed(&x.value),
            Insert::CryptoAddr(x) => Cow::Borrowed(&x.value),
        };
        Ok(value)
    }

    #[inline]
    pub fn table(&self) -> &str {
        Table::from(self).into()
    }

    #[inline]
    pub fn family(&self) -> &str {
        match self {
            Insert::Domain(_) => Family::Domain.as_str(),
            Insert::Subdomain(_) => Family::Subdomain.as_str(),
            Insert::IpAddr(_) => Family::Ipaddr.as_str(),
            Insert::SubdomainIpAddr(_) => Family::SubdomainIpaddr.as_str(),
            Insert::Url(_) => Family::Url.as_str(),
            Insert::Email(_) => Family::Email.as_str(),
            Insert::PhoneNumber(_) => Family::Phonenumber.as_str(),
            Insert::Device(_) => Family::Device.as_str(),
            Insert::Network(_) => Family::Network.as_str(),
            Insert::NetworkDevice(_) => Family::NetworkDevice.as_str(),
            Insert::Account(_) => Family::Account.as_str(),
            Insert::Breach(_) => Family::Breach.as_str(),
            Insert::BreachEmail(_) => Family::BreachEmail.as_str(),
            Insert::Image(_) => Family::Image.as_str(),
            Insert::Port(_) => Family::Port.as_str(),
            Insert::Netblock(_) => Family::Netblock.as_str(),
            Insert::CryptoAddr(_) => Family::Cryptoaddr.as_str(),
        }
    }
}

impl From<&Insert> for Table {
    fn from(insert: &Insert) -> Table {
        match insert {
            Insert::Domain(_) => Table::Domains,
            Insert::Subdomain(_) => Table::Subdomains,
            Insert::IpAddr(_) => Table::Ipaddrs,
            Insert::SubdomainIpAddr(_) => Table::SubdomainIpaddrs,
            Insert::Url(_) => Table::Urls,
            Insert::Email(_) => Table::Emails,
            Insert::PhoneNumber(_) => Table::Phonenumbers,
            Insert::Device(_) => Table::Devices,
            Insert::Network(_) => Table::Networks,
            Insert::NetworkDevice(_) => Table::NetworkDevices,
            Insert::Account(_) => Table::Accounts,
            Insert::Breach(_) => Table::Breaches,
            Insert::BreachEmail(_) => Table::BreachEmails,
            Insert::Image(_) => Table::Images,
            Insert::Port(_) => Table::Ports,
            Insert::Netblock(_) => Table::Netblocks,
            Insert::CryptoAddr(_) => Table::Cryptoaddrs,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Update {
    Subdomain(SubdomainUpdate),
    IpAddr(IpAddrUpdate),
    Url(UrlChangeset),
    Email(EmailUpdate),
    PhoneNumber(PhoneNumberUpdate),
    Device(DeviceUpdate),
    Network(NetworkUpdate),
    NetworkDevice(NetworkDeviceUpdate),
    Account(AccountUpdate),
    BreachEmail(BreachEmailUpdate),
    Image(ImageUpdate),
    Port(PortUpdate),
    Netblock(NetblockUpdate),
    CryptoAddr(CryptoAddrUpdate),
}

impl Update {
    pub fn is_dirty(&self) -> bool {
        match self {
            Update::Subdomain(update)     => update.is_dirty(),
            Update::IpAddr(update)        => update.is_dirty(),
            Update::Url(update)           => update.is_dirty(),
            Update::Email(update)         => update.is_dirty(),
            Update::PhoneNumber(update)   => update.is_dirty(),
            Update::Device(update)        => update.is_dirty(),
            Update::Network(update)       => update.is_dirty(),
            Update::NetworkDevice(update) => update.is_dirty(),
            Update::Account(update)       => update.is_dirty(),
            Update::BreachEmail(update)   => update.is_dirty(),
            Update::Image(update)         => update.is_dirty(),
            Update::Port(update)          => update.is_dirty(),
            Update::Netblock(update)      => update.is_dirty(),
            Update::CryptoAddr(update)    => update.is_dirty(),
        }
    }

    pub fn to_plain_str(&self) -> String {
        match self {
            Update::Subdomain(update)       => update.to_plain_str(),
            Update::IpAddr(update)          => update.to_plain_str(),
            Update::Url(update)             => update.to_plain_str(),
            Update::Email(update)           => update.to_plain_str(),
            Update::PhoneNumber(update)     => update.to_plain_str(),
            Update::Device(update)          => update.to_plain_str(),
            Update::Network(update)         => update.to_plain_str(),
            Update::NetworkDevice(update)   => update.to_plain_str(),
            Update::Account(update)         => update.to_plain_str(),
            Update::BreachEmail(update)     => update.to_plain_str(),
            Update::Image(update)           => update.to_plain_str(),
            Update::Port(update)            => update.to_plain_str(),
            Update::Netblock(update)        => update.to_plain_str(),
            Update::CryptoAddr(update)      => update.to_plain_str(),
        }
    }

    pub fn to_term_str(&self) -> String {
        match self {
            Update::Subdomain(update)       => update.to_term_str(),
            Update::IpAddr(update)          => update.to_term_str(),
            Update::Url(update)             => update.to_term_str(),
            Update::Email(update)           => update.to_term_str(),
            Update::PhoneNumber(update)     => update.to_term_str(),
            Update::Device(update)          => update.to_term_str(),
            Update::Network(update)         => update.to_term_str(),
            Update::NetworkDevice(update)   => update.to_term_str(),
            Update::Account(update)         => update.to_term_str(),
            Update::BreachEmail(update)     => update.to_term_str(),
            Update::Image(update)           => update.to_term_str(),
            Update::Port(update)            => update.to_term_str(),
            Update::Netblock(update)        => update.to_term_str(),
            Update::CryptoAddr(update)      => update.to_term_str(),
        }
    }
}

pub trait Model: Sized {
    type ID: ?Sized;

    fn to_string(&self) -> String;

    fn list(db: &Database) -> Result<Vec<Self>>;

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>>;

    fn filter_with_param(_db: &Database, _filter: &Filter, _param: &str) -> Result<Vec<Self>> {
        unimplemented!("This model doesn't support filtering with an additional parameter")
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize>;

    fn delete_id(db: &Database, my_id: i32) -> Result<usize>;

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

    fn blob(&self) -> Option<&str> {
        None
    }
}

pub trait Scopable: Model {
    fn scoped(&self) -> bool;

    fn set_scoped(&self, _db: &Database, _value: bool) -> Result<()>;

    fn scope(db: &Database, filter: &Filter) -> Result<usize>;

    fn noscope(db: &Database, filter: &Filter) -> Result<usize>;
}

pub trait InsertableStruct<T: Model>: Upsertable<T> {
    fn value(&self) -> &T::ID;

    fn set_scoped(&mut self, scoped: bool);

    fn insert(&self, db: &Database) -> Result<()>;
}

pub trait Upsertable<M> {
    type Update: Upsert;

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
    fn to_plain_str(&self) -> String {
        let mut updates = Vec::new();
        self.fmt(&mut updates, false);
        updates.join(", ")
    }

    fn to_term_str(&self) -> String {
        let mut updates = Vec::new();
        self.fmt(&mut updates, true);
        updates.join(", ")
    }

    #[inline]
    fn clear_if_equal<T: PartialEq>(update: &mut Option<T>, existing: &Option<T>) {
        if update == existing { update.take(); }
    }

    fn clear_if_lower_or_equal<T: PartialOrd>(update: &mut Option<T>, existing: &Option<T>) {
        if let (Some(new), Some(old)) = (&update, &existing) {
            if *new <= *old {
                update.take();
            }
        }
    }

    fn clear_if_greater_or_equal<T: PartialOrd>(update: &mut Option<T>, existing: &Option<T>) {
        if let (Some(new), Some(old)) = (&update, &existing) {
            if *new >= *old {
                update.take();
            }
        }
    }

    fn changeset(&mut self, existing: &M);

    #[inline]
    fn push_value<D: fmt::Debug>(updates: &mut Vec<String>, name: &str, value: &Option<D>, colors: bool) {
        if let Some(v) = value {
            if colors {
                updates.push(format!("{} => \x1b[33m{:?}\x1b[0m", name, v));
            } else {
                updates.push(format!("{} => {:?}", name, v));
            }
        }
    }

    #[inline]
    fn push_raw<T: AsRef<str>>(updates: &mut Vec<String>, name: &str, value: Option<T>, colors: bool) {
        if let Some(v) = value {
            if colors {
                updates.push(format!("{} => \x1b[33m{}\x1b[0m", name, v.as_ref()));
            } else {
                updates.push(format!("{} => {}", name, v.as_ref()));
            }
        }
    }

    fn fmt(&self, updates: &mut Vec<String>, colors: bool);
}

// TODO: Printable could probably be dropped
pub trait Printable<T: Sized> {
    fn printable(&self, db: &Database) -> Result<T>;
}

pub trait Detailed: Scopable {
    type T: fmt::Display;

    fn detailed(&self, db: &Database) -> Result<Self::T>;
}

pub trait DisplayableDetailed {
    fn scoped(&self) -> bool;

    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result;

    fn children(&self, w: &mut fmt::DetailFormatter) -> fmt::Result;
}

macro_rules! display_detailed {
    ( $name:ident ) => {
        impl fmt::Display for $name {
            fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
                let mut w = fmt::DetailFormatter::new(w, self.scoped());
                w.start()?;
                self.print(&mut w)?;
                w.end()?;
                self.children(&mut w)?;
                Ok(())
            }
        }
    };
}

pub trait LuaInsertToNew {
    type Target;

    fn lua_try_into_new(self, state: &Arc<dyn State>) -> Result<Self::Target>;
}

pub trait InsertToNew {
    type Target;

    fn try_into_new(self) -> Result<Self::Target>;
}

impl<T: InsertToNew> LuaInsertToNew for T {
    type Target = T::Target;

    #[inline]
    fn lua_try_into_new(self, _state: &Arc<dyn State>) -> Result<Self::Target> {
        self.try_into_new()
    }
}

pub trait UpdateToChangeset<T> {
    fn try_into_changeset(self) -> Result<T>;
}

mod domain;
pub use self::domain::*;

mod subdomain;
pub use self::subdomain::*;

mod netblock;
pub use self::netblock::*;

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

mod network;
pub use self::network::*;

mod device;
pub use self::device::*;

mod network_device;
pub use self::network_device::*;

mod account;
pub use self::account::*;

mod breach;
pub use self::breach::*;

mod breach_email;
pub use self::breach_email::*;

mod image;
pub use self::image::*;

mod port;
pub use self::port::*;

mod cryptoaddr;
pub use self::cryptoaddr::*;

mod activity;
pub use self::activity::*;
