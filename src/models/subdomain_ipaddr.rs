use crate::errors::*;
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use crate::models::*;
use std::net;

#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[belongs_to(Subdomain)]
#[belongs_to(IpAddr)]
#[table_name="subdomain_ipaddrs"]
pub struct SubdomainIpAddr {
    pub id: i32,
    pub subdomain_id: i32,
    pub ip_addr_id: i32,
}

impl Model for SubdomainIpAddr {
    type ID = (i32, i32);

    fn to_string(&self) -> String {
        unimplemented!("SubdomainIpAddr can not be printed")
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::subdomain_ipaddrs::dsl::*;

        let results = subdomain_ipaddrs.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::subdomain_ipaddrs::dsl::*;

        let query = subdomain_ipaddrs.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::subdomain_ipaddrs::dsl::*;

        diesel::delete(subdomain_ipaddrs.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::subdomain_ipaddrs::dsl::*;

        diesel::delete(subdomain_ipaddrs.filter(id.eq(my_id)))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn id(&self) -> i32 {
        self.id
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use crate::schema::subdomain_ipaddrs::dsl::*;

        let subdomain_ipaddr = subdomain_ipaddrs.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(subdomain_ipaddr)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::subdomain_ipaddrs::dsl::*;

        let (my_subdomain_id, my_ip_addr_id) = query;
        let subdomain_ipaddr = subdomain_ipaddrs.filter(subdomain_id.eq(my_subdomain_id))
                                                   .filter(ip_addr_id.eq(my_ip_addr_id))
                                                   .first::<Self>(db.db())?;

        Ok(subdomain_ipaddr)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::subdomain_ipaddrs::dsl::*;

        let (my_subdomain_id, my_ip_addr_id) = query;
        let subdomain_ipaddr = subdomain_ipaddrs.filter(subdomain_id.eq(my_subdomain_id))
                                                   .filter(ip_addr_id.eq(my_ip_addr_id))
                                                   .first::<Self>(db.db())
                                                   .optional()?;

        Ok(subdomain_ipaddr)
    }
}

pub struct PrintableSubdomainIpAddr {
    subdomain: String,
    ipaddr: net::IpAddr,
}

impl fmt::Display for PrintableSubdomainIpAddr {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?} -> {}", self.subdomain, self.ipaddr)
    }
}

impl Printable<PrintableSubdomainIpAddr> for SubdomainIpAddr {
    fn printable(&self, db: &Database) -> Result<PrintableSubdomainIpAddr> {
        let subdomain = Subdomain::by_id(db, self.subdomain_id)?;
        let ipaddr = IpAddr::by_id(db, self.ip_addr_id)?;
        Ok(PrintableSubdomainIpAddr {
            subdomain: subdomain.value,
            ipaddr: ipaddr.value.parse()?,
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="subdomain_ipaddrs"]
pub struct NewSubdomainIpAddr {
    pub subdomain_id: i32,
    pub ip_addr_id: i32,
}

impl Printable<PrintableSubdomainIpAddr> for NewSubdomainIpAddr {
    fn printable(&self, db: &Database) -> Result<PrintableSubdomainIpAddr> {
        let subdomain = Subdomain::by_id(db, self.subdomain_id)?;
        let ipaddr = IpAddr::by_id(db, self.ip_addr_id)?;
        Ok(PrintableSubdomainIpAddr {
            subdomain: subdomain.value,
            ipaddr: ipaddr.value.parse()?,
        })
    }
}

pub type InsertSubdomainIpAddr = NewSubdomainIpAddr;

impl InsertToNew for InsertSubdomainIpAddr {
    type Target = NewSubdomainIpAddr;

    #[inline]
    fn try_into_new(self) -> Result<NewSubdomainIpAddr> {
        Ok(self)
    }
}
