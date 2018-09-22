use errors::*;
use diesel::prelude::*;
use json::LuaJsonValue;
use models::*;
use serde_json;
use std::net;


#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="ipaddrs"]
pub struct IpAddr {
    pub id: i32,
    pub family: String,
    pub value: String,
}

impl Model for IpAddr {
    type ID = str;

    fn list(db: &Database) -> Result<Vec<Self>> {
        use schema::ipaddrs::dsl::*;

        let results = ipaddrs.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use schema::ipaddrs::dsl::*;

        let query = ipaddrs.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use schema::ipaddrs::dsl::*;

        let ipaddr = ipaddrs.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(ipaddr)
    }

    fn id(db: &Database, query: &Self::ID) -> Result<i32> {
        use schema::ipaddrs::dsl::*;

        let ipaddr_id = ipaddrs.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())?;

        Ok(ipaddr_id)
    }

    fn id_opt(db: &Database, query: &Self::ID) -> Result<Option<i32>> {
        use schema::ipaddrs::dsl::*;

        let ipaddr_id = ipaddrs.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())
            .optional()?;

        Ok(ipaddr_id)
    }
}

pub struct PrintableIpAddr {
    value: net::IpAddr,
}

impl fmt::Display for PrintableIpAddr {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{}", self.value)
    }
}

impl Printable<PrintableIpAddr> for IpAddr {
    fn printable(&self, _db: &Database) -> Result<PrintableIpAddr> {
        Ok(PrintableIpAddr {
            value: self.value.parse()?,
        })
    }
}

impl Detailed for IpAddr {
    type T = PrintableIpAddr;

    fn detailed(&self, db: &Database) -> Result<Self::T> {
        self.printable(db)
    }
}

#[derive(Insertable)]
#[table_name="ipaddrs"]
pub struct NewIpAddr<'a> {
    pub family: &'a str,
    pub value: &'a str,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="ipaddrs"]
pub struct NewIpAddrOwned {
    pub family: String,
    pub value: String,
}

impl NewIpAddrOwned {
    pub fn from_lua(x: LuaJsonValue) -> Result<NewIpAddrOwned> {
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

impl Printable<PrintableIpAddr> for NewIpAddrOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableIpAddr> {
        Ok(PrintableIpAddr {
            value: self.value.parse()?,
        })
    }
}
