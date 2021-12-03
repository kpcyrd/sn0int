use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::fmt::Write;
use crate::fmt::colors::*;
use crate::models::*;
use diesel::prelude::*;
use std::net;
use std::result;

#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="ipaddrs"]
pub struct IpAddr {
    pub id: i32,
    pub family: String,
    pub value: String,
    pub unscoped: bool,
    pub continent: Option<String>,
    pub continent_code: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub asn: Option<i32>,
    pub as_org: Option<String>,
    pub description: Option<String>,
    pub reverse_dns: Option<String>,
}

impl Model for IpAddr {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::ipaddrs::dsl::*;

        let results = ipaddrs.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::ipaddrs::dsl::*;

        let query = ipaddrs.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::ipaddrs::dsl::*;

        diesel::delete(ipaddrs.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::ipaddrs::dsl::*;

        diesel::delete(ipaddrs.filter(id.eq(my_id)))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn id(&self) -> i32 {
        self.id
    }

    fn value(&self) -> &Self::ID {
        &self.value
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use crate::schema::ipaddrs::dsl::*;

        let ipaddr = ipaddrs.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(ipaddr)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::ipaddrs::dsl::*;

        let ipaddr = ipaddrs.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(ipaddr)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::ipaddrs::dsl::*;

        let ipaddr = ipaddrs.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(ipaddr)
    }
}

impl Scopable for IpAddr {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn set_scoped(&self, db: &Database, my_value: bool) -> Result<()> {
        use crate::schema::ipaddrs::dsl::*;
        diesel::update(ipaddrs.filter(id.eq(self.id)))
            .set(unscoped.eq(!my_value))
            .execute(db.db())?;
        Ok(())
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::ipaddrs::dsl::*;

        diesel::update(ipaddrs.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::ipaddrs::dsl::*;

        diesel::update(ipaddrs.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

impl IpAddr {
    fn subdomains(&self, db: &Database) -> Result<Vec<Subdomain>> {
        let subdomain_ids = SubdomainIpAddr::belonging_to(self)
            .select(subdomain_ipaddrs::subdomain_id)
            .load::<i32>(db.db())?;

        subdomain_ids.into_iter()
            .map(|subdomain_id| subdomains::table
                .filter(subdomains::id.eq(subdomain_id))
                .first::<Subdomain>(db.db())
            )
            .collect::<result::Result<_, _>>()
            .map_err(Error::from)
    }

    fn ports(&self, db: &Database) -> Result<Vec<Port>> {
        Port::belonging_to(self)
            .load(db.db())
            .map_err(Error::from)
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

pub struct DetailedIpAddr {
    id: i32,
    value: net::IpAddr,
    subdomains: Vec<PrintableSubdomain>,
    ports: Vec<PrintablePort>,
    unscoped: bool,
    continent: Option<String>,
    country: Option<String>,
    city: Option<String>,
    asn: Option<i32>,
    as_org: Option<String>,
    description: Option<String>,
    reverse_dns: Option<String>,
}

impl DisplayableDetailed for DetailedIpAddr {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        w.id(self.id)?;
        w.display::<Green, _>(&self.value)?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.description)?;
        w.opt_debug::<Yellow, _>(&self.reverse_dns)?;
        w.end_group()?;

        if let Some(ref continent) = self.continent {
            write!(w, " [{}", continent)?;

            if let Some(ref country) = self.country {
                write!(w, " / {}", country)?;
            }

            if let Some(ref city) = self.city {
                write!(w, " / {}", city)?;
            }

            write!(w, "]")?;
        }

        if let Some(ref asn) = self.asn {
            write!(w, " [{}", asn)?;

            if let Some(ref as_org) = self.as_org {
                write!(w, " / {:?}", as_org)?;
            }

            write!(w, "]")?;
        }

        Ok(())
    }

    #[inline]
    fn children(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        for subdomain in &self.subdomains {
            w.child(subdomain)?;
        }
        for port in &self.ports {
            w.child(port)?;
        }
        Ok(())
    }
}

display_detailed!(DetailedIpAddr);

impl Detailed for IpAddr {
    type T = DetailedIpAddr;

    fn detailed(&self, db: &Database) -> Result<Self::T> {
        let subdomains = self.subdomains(db)?.into_iter()
            .map(|x| x.printable(db))
            .collect::<Result<_>>()?;

        let ports = self.ports(db)?.into_iter()
            .map(|x| x.printable(db))
            .collect::<Result<_>>()?;

        Ok(DetailedIpAddr {
            id: self.id,
            value: self.value.parse()?,
            subdomains,
            ports,
            unscoped: self.unscoped,
            continent: self.continent.clone(),
            country: self.country.clone(),
            city: self.city.clone(),
            asn: self.asn,
            as_org: self.as_org.clone(),
            description: self.description.clone(),
            reverse_dns: self.reverse_dns.clone(),
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="ipaddrs"]
pub struct NewIpAddr {
    pub family: String,
    pub value: String,
    pub continent: Option<String>,
    pub continent_code: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub asn: Option<i32>,
    pub as_org: Option<String>,
    pub description: Option<String>,
    pub reverse_dns: Option<String>,

    pub unscoped: bool,
}

impl InsertableStruct<IpAddr> for NewIpAddr {
    fn value(&self) -> &str {
        &self.value
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.unscoped = !scoped;
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(ipaddrs::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl Upsertable<IpAddr> for NewIpAddr {
    type Update = IpAddrUpdate;

    fn upsert(self, existing: &IpAddr) -> Self::Update {
        Self::Update {
            id: existing.id,
            continent: Self::upsert_opt(self.continent, &existing.continent),
            continent_code: Self::upsert_opt(self.continent_code, &existing.continent_code),
            country: Self::upsert_opt(self.country, &existing.country),
            country_code: Self::upsert_opt(self.country_code, &existing.country_code),
            city: Self::upsert_opt(self.city, &existing.city),
            latitude: Self::upsert_opt(self.latitude, &existing.latitude),
            longitude: Self::upsert_opt(self.longitude, &existing.longitude),
            asn: Self::upsert_opt(self.asn, &existing.asn),
            as_org: Self::upsert_opt(self.as_org, &existing.as_org),
            description: Self::upsert_opt(self.description, &existing.description),
            reverse_dns: Self::upsert_opt(self.reverse_dns, &existing.reverse_dns),
        }
    }
}

impl Printable<PrintableIpAddr> for NewIpAddr {
    fn printable(&self, _db: &Database) -> Result<PrintableIpAddr> {
        Ok(PrintableIpAddr {
            value: self.value.parse()?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertIpAddr {
    // TODO: deprecate family field
    pub family: Option<String>,
    pub value: String,

    pub continent: Option<String>,
    pub continent_code: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub asn: Option<i32>,
    pub as_org: Option<String>,
    pub description: Option<String>,
    pub reverse_dns: Option<String>,
}

impl InsertToNew for InsertIpAddr {
    type Target = NewIpAddr;

    fn try_into_new(self) -> Result<NewIpAddr> {
        let ipaddr = self.value.parse::<net::IpAddr>()
            .context("Failed to parse ip address")?;

        let family = match ipaddr {
            net::IpAddr::V4(_) => String::from("4"),
            net::IpAddr::V6(_) => String::from("6"),
        };

        Ok(NewIpAddr {
            family,
            value: ipaddr.to_string(),

            continent: self.continent,
            continent_code: self.continent_code,
            country: self.country,
            country_code: self.country_code,
            city: self.city,
            latitude: self.latitude,
            longitude: self.longitude,
            asn: self.asn,
            as_org: self.as_org,
            description: self.description,
            reverse_dns: self.reverse_dns,

            unscoped: false,
        })
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="ipaddrs"]
pub struct IpAddrUpdate {
    pub id: i32,
    pub continent: Option<String>,
    pub continent_code: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub asn: Option<i32>,
    pub as_org: Option<String>,
    pub description: Option<String>,
    pub reverse_dns: Option<String>,
}

impl Upsert for IpAddrUpdate {
    fn is_dirty(&self) -> bool {
        self.continent.is_some() ||
        self.continent_code.is_some() ||
        self.country.is_some() ||
        self.country_code.is_some() ||
        self.city.is_some() ||
        self.latitude.is_some() ||
        self.longitude.is_some() ||
        self.asn.is_some() ||
        self.as_org.is_some() ||
        self.description.is_some() ||
        self.reverse_dns.is_some()
    }

    fn generic(self) -> Update {
        Update::IpAddr(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_ipaddr(self)
    }
}

impl Updateable<IpAddr> for IpAddrUpdate {
    fn changeset(&mut self, existing: &IpAddr) {
        Self::clear_if_equal(&mut self.continent, &existing.continent);
        Self::clear_if_equal(&mut self.continent_code, &existing.continent_code);
        Self::clear_if_equal(&mut self.country, &existing.country);
        Self::clear_if_equal(&mut self.country_code, &existing.country_code);
        Self::clear_if_equal(&mut self.city, &existing.city);
        Self::clear_if_equal(&mut self.latitude, &existing.latitude);
        Self::clear_if_equal(&mut self.longitude, &existing.longitude);
        Self::clear_if_equal(&mut self.asn, &existing.asn);
        Self::clear_if_equal(&mut self.as_org, &existing.as_org);
        Self::clear_if_equal(&mut self.description, &existing.description);
        Self::clear_if_equal(&mut self.reverse_dns, &existing.reverse_dns);
    }

    fn fmt(&self, updates: &mut Vec<String>, colors: bool) {
        Self::push_value(updates, "continent", &self.continent, colors);
        Self::push_value(updates, "continent_code", &self.continent_code, colors);
        Self::push_value(updates, "country", &self.country, colors);
        Self::push_value(updates, "country_code", &self.country_code, colors);
        Self::push_value(updates, "city", &self.city, colors);
        Self::push_value(updates, "latitude", &self.latitude, colors);
        Self::push_value(updates, "longitude", &self.longitude, colors);
        Self::push_value(updates, "asn", &self.asn, colors);
        Self::push_value(updates, "as_org", &self.as_org, colors);
        Self::push_value(updates, "description", &self.description, colors);
        Self::push_value(updates, "reverse_dns", &self.reverse_dns, colors);
    }
}
