use errors::*;
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
