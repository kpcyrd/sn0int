use std::fmt;
use schema::*;


#[derive(Debug, Serialize, Deserialize)]
pub enum Object {
    Subdomain(NewSubdomainOwned),
    IpAddr(NewIpAddrOwned),
    SubdomainIpAddr(NewSubdomainIpAddr),
    Url(NewUrlOwned),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Subdomain(x) => write!(f, "Subdomain: {:?}", x.value),
            Object::IpAddr(x) => write!(f, "IpAddr: {:?}", x.value),
            Object::SubdomainIpAddr(x) => write!(f, "Subdomain->IpAddr: {}->{}", x.subdomain_id, x.ip_addr_id),
            Object::Url(x) => {
                write!(f, "Url: {:?}", x.value)?;
                if let Some(status) = x.status {
                    write!(f, " ({})", status)?;
                }
                Ok(())
            },
        }
    }
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
