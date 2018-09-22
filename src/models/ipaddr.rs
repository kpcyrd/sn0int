use errors::*;
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
