use crate::errors::*;
use diesel;
use diesel::prelude::*;
use crate::models::*;
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
    unscoped: bool,
    continent: Option<String>,
    country: Option<String>,
    city: Option<String>,
    asn: Option<i32>,
    as_org: Option<String>,
}

impl DisplayableDetailed for DetailedIpAddr {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.id(w, self.id)?;
        self.green_debug(w, &self.value)?;

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
    fn children(&self, w: &mut fmt::Formatter) -> fmt::Result {
        for subdomain in &self.subdomains {
            self.child(w, subdomain)?;
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

        Ok(DetailedIpAddr {
            id: self.id,
            value: self.value.parse()?,
            subdomains,
            unscoped: self.unscoped,
            continent: self.continent.clone(),
            country: self.country.clone(),
            city: self.city.clone(),
            asn: self.asn,
            as_org: self.as_org.clone(),
        })
    }
}

#[derive(Insertable)]
#[table_name="ipaddrs"]
pub struct NewIpAddr<'a> {
    pub family: &'a str,
    pub value: &'a str,
    pub continent: Option<&'a String>,
    pub continent_code: Option<&'a String>,
    pub country: Option<&'a String>,
    pub country_code: Option<&'a String>,
    pub city: Option<&'a String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub asn: Option<i32>,
    pub as_org: Option<&'a String>,
}

impl<'a> InsertableStruct<IpAddr> for NewIpAddr<'a> {
    fn value(&self) -> &str {
        self.value
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(ipaddrs::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl<'a> Upsertable<IpAddr> for NewIpAddr<'a> {
    type Update = IpAddrUpdate;

    fn upsert(self, existing: &IpAddr) -> Self::Update {
        Self::Update {
            id: existing.id,
            continent: Self::upsert_str(self.continent, &existing.continent),
            continent_code: Self::upsert_str(self.continent_code, &existing.continent_code),
            country: Self::upsert_str(self.country, &existing.country),
            country_code: Self::upsert_str(self.country_code, &existing.country_code),
            city: Self::upsert_str(self.city, &existing.city),
            latitude: Self::upsert_opt(self.latitude, &existing.latitude),
            longitude: Self::upsert_opt(self.longitude, &existing.longitude),
            asn: Self::upsert_opt(self.asn, &existing.asn),
            as_org: Self::upsert_str(self.as_org, &existing.as_org),
        }
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="ipaddrs"]
pub struct NewIpAddrOwned {
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
}

impl Printable<PrintableIpAddr> for NewIpAddrOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableIpAddr> {
        Ok(PrintableIpAddr {
            value: self.value.parse()?,
        })
    }
}

pub type InsertIpAddr = NewIpAddrOwned;

impl LuaInsertToNewOwned for InsertIpAddr {
    type Target = NewIpAddrOwned;

    fn try_into_new(self) -> Result<NewIpAddrOwned> {
        Ok(self)
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
        self.as_org.is_some()
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
    }

    fn fmt(&self, updates: &mut Vec<String>) {
        Self::push_value(updates, "continent", &self.continent);
        Self::push_value(updates, "continent_code", &self.continent_code);
        Self::push_value(updates, "country", &self.country);
        Self::push_value(updates, "country_code", &self.country_code);
        Self::push_value(updates, "city", &self.city);
        Self::push_value(updates, "latitude", &self.latitude);
        Self::push_value(updates, "longitude", &self.longitude);
        Self::push_value(updates, "asn", &self.asn);
        Self::push_value(updates, "as_org", &self.as_org);
    }
}
