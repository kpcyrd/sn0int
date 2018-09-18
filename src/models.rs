use errors::*;
use json::LuaJsonValue;
use std::fmt;
use serde_json;
use schema::*;
// use std::convert::AsRef;


#[derive(Debug, Serialize, Deserialize)]
pub enum Object {
    Subdomain(NewSubdomainOwned),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Subdomain(x) => write!(f, "Subdomain: {:?}", x.value),
        }
    }
}

#[derive(Identifiable, Queryable, Serialize, PartialEq, Debug)]
#[table_name="domains"]
pub struct Domain {
    pub id: i32,
    pub value: String,
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Insertable)]
#[table_name="domains"]
pub struct NewDomain<'a> {
    pub value: &'a str,
}

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

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="ipaddrs"]
pub struct IpAddr {
    pub id: i32,
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Subdomain)]
#[belongs_to(IpAddr)]
#[table_name="subdomain_ipaddrs"]
pub struct SubdomainIpAddr {
    pub id: i32,
    pub subdomain_id: i32,
    pub ip_addr_id: i32,
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Subdomain)]
#[table_name="urls"]
pub struct Url {
    pub id: i32,
    pub subdomain_id: i32,
    pub status: u16,
    pub body: Vec<u8>,
}
