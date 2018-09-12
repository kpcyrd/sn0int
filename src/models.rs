use schema::*;

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name="domains"]
pub struct Domain {
    pub id: i64,
    pub value: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Domain)]
#[table_name="subdomains"]
pub struct Subdomain {
    pub id: i64,
    pub domain_id: i64,
    pub value: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="ipaddrs"]
pub struct IpAddr {
    pub id: i64,
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Subdomain)]
#[belongs_to(IpAddr)]
#[table_name="subdomain_ipaddrs"]
pub struct SubdomainIpAddr {
    pub id: i64,
    pub subdomain_id: i64,
    pub ip_addr_id: i64,
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Subdomain)]
#[table_name="urls"]
pub struct Url {
    pub id: i64,
    pub subdomain_id: i64,
    pub status: u16,
    pub body: Vec<u8>,
}
