use errors::*;
use diesel::prelude::*;
use json::LuaJsonValue;
use models::*;
use serde_json;


#[derive(Identifiable, Queryable, Associations)]
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

    fn list(db: &Database) -> Result<Vec<Self>> {
        use schema::subdomain_ipaddrs::dsl::*;

        let results = subdomain_ipaddrs.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use schema::subdomain_ipaddrs::dsl::*;

        let query = subdomain_ipaddrs.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn id(db: &Database, query: &Self::ID) -> Result<i32> {
        use schema::subdomain_ipaddrs::dsl::*;

        let (my_subdomain_id, my_ip_addr_id) = query;
        let subdomain_ipaddr_id = subdomain_ipaddrs.filter(subdomain_id.eq(my_subdomain_id))
                                                   .filter(ip_addr_id.eq(my_ip_addr_id))
                                                   .select(id)
                                                   .first::<i32>(db.db())?;

        Ok(subdomain_ipaddr_id)
    }

    fn id_opt(db: &Database, query: &Self::ID) -> Result<Option<i32>> {
        use schema::subdomain_ipaddrs::dsl::*;

        let (my_subdomain_id, my_ip_addr_id) = query;
        let subdomain_ipaddr_id = subdomain_ipaddrs.filter(subdomain_id.eq(my_subdomain_id))
                                                   .filter(ip_addr_id.eq(my_ip_addr_id))
                                                   .select(id)
                                                   .first::<i32>(db.db())
                                                   .optional()?;

        Ok(subdomain_ipaddr_id)
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="subdomain_ipaddrs"]
pub struct NewSubdomainIpAddr {
    pub subdomain_id: i32,
    pub ip_addr_id: i32,
}

impl NewSubdomainIpAddr {
    pub fn from_lua(x: LuaJsonValue) -> Result<NewSubdomainIpAddr> {
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}
