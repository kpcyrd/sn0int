use errors::*;
use diesel::prelude::*;
use json::LuaJsonValue;
use models::*;
use serde_json;
use std::net;


#[derive(Identifiable, Queryable, Associations, Serialize, PartialEq, Debug)]
#[belongs_to(Domain)]
#[table_name="subdomains"]
pub struct Subdomain {
    pub id: i32,
    pub domain_id: i32,
    pub value: String,
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
    value: String,
    ips: Vec<net::IpAddr>,
}

impl fmt::Display for DetailedSubdomain {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)?;
        if !self.ips.is_empty() {
            write!(w, " (TODO)")?;
        }
        Ok(())
    }
}

impl Detailed for Subdomain {
    type T = DetailedSubdomain;

    fn detailed(&self, _db: &Database) -> Result<Self::T> {
        Ok(DetailedSubdomain {
            value: self.value.to_string(),
            ips: vec![],
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
