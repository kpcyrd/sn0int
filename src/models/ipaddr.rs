use errors::*;
use diesel::prelude::*;
use json::LuaJsonValue;
use models::*;
use serde_json;


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
