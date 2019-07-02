use crate::errors::*;
use crate::db::{Database, Filter};
use crate::fmt;
use crate::schema::*;
use std::sync::Arc;
use crate::engine::ctx::State;


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
}

impl Insert {
    pub fn label(&self, db: &Database) -> Result<String> {
        let label = match self {
            Insert::Domain(x) => format!("{:?}", x.value),
            Insert::Subdomain(x) => format!("{:?}", x.value),
            Insert::IpAddr(x) => format!("{:?}", x.value),
            Insert::SubdomainIpAddr(x) => {
                let subdomain = Subdomain::by_id(db, x.subdomain_id)?;
                let ipaddr = IpAddr::by_id(db, x.ip_addr_id)?;
                format!("{:?}+{:?}", subdomain.value, ipaddr.value)
            },
            Insert::Url(x) => format!("{:?}", x.value),
            Insert::Email(x) => format!("{:?}", x.value),
            Insert::PhoneNumber(x) => format!("{:?}", x.value),
            Insert::Device(x) => format!("{:?}", x.value),
            Insert::Network(x) => format!("{:?}", x.value),
            Insert::NetworkDevice(x) => {
                let network = Network::by_id(db, x.network_id)?;
                let device = Device::by_id(db, x.device_id)?;
                format!("{:?}+{:?}", network.value, device.value)
            },
            Insert::Account(x) => format!("{:?}", x.value),
            Insert::Breach(x) => format!("{:?}", x.value),
            Insert::BreachEmail(x) => {
                let breach = Breach::by_id(db, x.breach_id)?;
                let email = Email::by_id(db, x.email_id)?;
                format!("{:?}+{:?}", breach.value, email.value)
            }
            Insert::Image(x) => format!("{:?}", x.value),
            Insert::Port(x) => format!("{:?}", x.value),
            Insert::Netblock(x) => format!("{:?}", x.value),
        };
        Ok(label)
    }

    pub fn table(&self) -> &str {
        match self {
            Insert::Domain(_) => "domains",
            Insert::Subdomain(_) => "subdomains",
            Insert::IpAddr(_) => "ipaddrs",
            Insert::SubdomainIpAddr(_) => "subdomain_ipaddrs",
            Insert::Url(_) => "urls",
            Insert::Email(_) => "emails",
            Insert::PhoneNumber(_) => "phonenumbers",
            Insert::Device(_) => "devices",
            Insert::Network(_) => "networks",
            Insert::NetworkDevice(_) => "network_devices",
            Insert::Account(_) => "accounts",
            Insert::Breach(_) => "breaches",
            Insert::BreachEmail(_) => "breach_emails",
            Insert::Image(_) => "images",
            Insert::Port(_) => "ports",
            Insert::Netblock(_) => "netblocks",
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
            Insert::PhoneNumber(x) => format!("PhoneNumber: {}", x.printable(db)?),
            Insert::Device(x) => format!("Device: {}", x.printable(db)?),
            Insert::Network(x) => format!("Network: {}", x.printable(db)?),
            Insert::NetworkDevice(x) => x.printable(db)?.to_string(),
            Insert::Account(x) => format!("Account: {}", x.printable(db)?),
            Insert::Breach(x) => format!("Breach: {}", x.printable(db)?),
            Insert::BreachEmail(x) => x.printable(db)?.to_string(),
            Insert::Image(x) => format!("Image: {}", x.printable(db)?),
            Insert::Port(x) => format!("Port: {}", x.printable(db)?),
            Insert::Netblock(x) => format!("Netblock: {}", x.printable(db)?),
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
    Device(DeviceUpdate),
    Network(NetworkUpdate),
    NetworkDevice(NetworkDeviceUpdate),
    Account(AccountUpdate),
    BreachEmail(BreachEmailUpdate),
    Image(ImageUpdate),
    Port(PortUpdate),
    Netblock(NetblockUpdate),
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
        }
    }
}

impl fmt::Display for Update {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Update::Subdomain(update)     => write!(w, "{}", update.to_string()),
            Update::IpAddr(update)        => write!(w, "{}", update.to_string()),
            Update::Url(update)           => write!(w, "{}", update.to_string()),
            Update::Email(update)         => write!(w, "{}", update.to_string()),
            Update::PhoneNumber(update)   => write!(w, "{}", update.to_string()),
            Update::Device(update)        => write!(w, "{}", update.to_string()),
            Update::Network(update)       => write!(w, "{}", update.to_string()),
            Update::NetworkDevice(update) => write!(w, "{}", update.to_string()),
            Update::Account(update)       => write!(w, "{}", update.to_string()),
            Update::BreachEmail(update)   => write!(w, "{}", update.to_string()),
            Update::Image(update)         => write!(w, "{}", update.to_string()),
            Update::Port(update)          => write!(w, "{}", update.to_string()),
            Update::Netblock(update)      => write!(w, "{}", update.to_string()),
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

    fn try_into_new(self, state: &Arc<State>) -> Result<Self::Target>;
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
