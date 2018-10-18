use errors::*;
use diesel;
use diesel::prelude::*;
use json::LuaJsonValue;
use models::*;
use serde_json;
use std::result;


#[derive(Identifiable, Queryable, Associations, Serialize, PartialEq, Debug)]
#[belongs_to(Domain)]
#[table_name="subdomains"]
pub struct Subdomain {
    pub id: i32,
    pub domain_id: i32,
    pub value: String,
    pub unscoped: bool,
    pub resolvable: Option<bool>,
}

impl fmt::Display for Subdomain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Model for Subdomain {
    type ID = str;

    fn list(db: &Database) -> Result<Vec<Self>> {
        use schema::subdomains::dsl::*;

        let results = subdomains.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use schema::subdomains::dsl::*;

        let query = subdomains.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use schema::subdomains::dsl::*;

        let subdomain = subdomains.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(subdomain)
    }

    fn id(db: &Database, query: &Self::ID) -> Result<i32> {
        use schema::subdomains::dsl::*;

        let subdomain_id = subdomains.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())?;

        Ok(subdomain_id)
    }

    fn id_opt(db: &Database, query: &Self::ID) -> Result<Option<i32>> {
        use schema::subdomains::dsl::*;

        let subdomain_id = subdomains.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())
            .optional()?;

        Ok(subdomain_id)
    }
}

impl Scopable for Subdomain {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::subdomains::dsl::*;

        diesel::update(subdomains.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::subdomains::dsl::*;

        diesel::update(subdomains.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

impl Subdomain {
    fn ip_addrs(&self, db: &Database) -> Result<Vec<IpAddr>> {
        let ipaddr_ids = SubdomainIpAddr::belonging_to(self)
            .select(subdomain_ipaddrs::ip_addr_id)
            .load::<i32>(db.db())?;

        ipaddr_ids.into_iter()
            .map(|ipaddr_id| ipaddrs::table
                .filter(ipaddrs::id.eq(ipaddr_id))
                .first::<IpAddr>(db.db())
            )
            .collect::<result::Result<_, _>>()
            .map_err(Error::from)
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="subdomains"]
pub struct SubdomainUpdate {
    pub id: i32,
    pub resolvable: Option<bool>,
}

impl SubdomainUpdate {
    pub fn from_lua(x: LuaJsonValue) -> Result<Self> {
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

impl fmt::Display for SubdomainUpdate {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if let Some(resolvable) = self.resolvable {
            write!(w, "resolvable => {:?}", resolvable)?;
        }
        Ok(())
    }
}

pub struct PrintableSubdomain {
    value: String,
}

impl fmt::Display for PrintableSubdomain {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintableSubdomain> for Subdomain {
    fn printable(&self, _db: &Database) -> Result<PrintableSubdomain> {
        Ok(PrintableSubdomain {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedSubdomain {
    id: i32,
    value: String,
    ipaddrs: Vec<PrintableIpAddr>,
    unscoped: bool,
}

// TODO: maybe print urls as well
impl fmt::Display for DetailedSubdomain {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if !self.unscoped {
            write!(w, "\x1b[32m#{}\x1b[0m, \x1b[32m{:?}\x1b[0m", self.id, self.value)?;

            for ipaddr in &self.ipaddrs {
                write!(w, "\n\t\x1b[33m{}\x1b[0m", ipaddr)?;
            }
        } else {
            write!(w, "\x1b[90m#{}, {:?}\x1b[0m", self.id, self.value)?;

            for ipaddr in &self.ipaddrs {
                write!(w, "\n\t\x1b[90m{}\x1b[0m", ipaddr)?;
            }
        }

        Ok(())
    }
}

impl Detailed for Subdomain {
    type T = DetailedSubdomain;

    fn detailed(&self, db: &Database) -> Result<Self::T> {
        let ipaddrs = self.ip_addrs(db)?.into_iter()
            .map(|ip| ip.printable(db))
            .collect::<Result<_>>()?;

        Ok(DetailedSubdomain {
            id: self.id,
            value: self.value.to_string(),
            ipaddrs,
            unscoped: self.unscoped,
        })
    }
}

#[derive(Insertable)]
#[table_name="subdomains"]
pub struct NewSubdomain<'a> {
    pub domain_id: i32,
    pub value: &'a str,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="subdomains"]
pub struct NewSubdomainOwned {
    pub domain_id: i32,
    pub value: String,
}

impl NewSubdomainOwned {
    pub fn from_lua(x: LuaJsonValue) -> Result<NewSubdomainOwned> {
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

impl Printable<PrintableSubdomain> for NewSubdomainOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableSubdomain> {
        Ok(PrintableSubdomain {
            value: self.value.to_string(),
        })
    }
}
