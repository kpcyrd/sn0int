use errors::*;
use diesel;
use diesel::prelude::*;
use models::*;
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

impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
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

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::ipaddrs::dsl::*;

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
        use schema::ipaddrs::dsl::*;

        let ipaddr = ipaddrs.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(ipaddr)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use schema::ipaddrs::dsl::*;

        let ipaddr = ipaddrs.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(ipaddr)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use schema::ipaddrs::dsl::*;

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
        use schema::ipaddrs::dsl::*;

        diesel::update(ipaddrs.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::ipaddrs::dsl::*;

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

impl fmt::Display for DetailedIpAddr {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if !self.unscoped {
            write!(w, "\x1b[32m#{}\x1b[0m, \x1b[32m{}\x1b[0m", self.id, self.value)?;

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

            for subdomain in &self.subdomains {
                write!(w, "\n\t\x1b[33m{}\x1b[0m", subdomain)?;
            }
        } else {
            write!(w, "\x1b[90m#{}, {}", self.id, self.value)?;

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

            write!(w, "\x1b[0m");

            for subdomain in &self.subdomains {
                write!(w, "\n\t\x1b[90m{}\x1b[0m", subdomain)?;
            }
        }

        Ok(())
    }
}

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

    fn upsert(&self, existing: &IpAddr) -> Self::Update {
        Self::Update {
            id: existing.id,
            continent: if self.continent != existing.continent.as_ref() { self.continent.map(|x| x.to_owned()) } else { None },
            continent_code: if self.continent_code != existing.continent_code.as_ref() { self.continent_code.map(|x| x.to_owned()) } else { None },
            country: if self.country != existing.country.as_ref() { self.country.map(|x| x.to_owned()) } else { None },
            country_code: if self.country_code != existing.country_code.as_ref() { self.country_code.map(|x| x.to_owned()) } else { None },
            city: if self.city != existing.city.as_ref() { self.city.map(|x| x.to_owned()) } else { None },
            latitude: if self.latitude != existing.latitude { self.latitude } else { None },
            longitude: if self.longitude != existing.longitude { self.longitude } else { None },
            asn: if self.asn != existing.asn { self.asn } else { None },
            as_org: if self.as_org != existing.as_org.as_ref() { self.as_org.map(|x| x.to_owned()) } else { None },
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

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_ipaddr(self)
    }
}

impl fmt::Display for IpAddrUpdate {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        let mut updates = Vec::new();

        if let Some(ref continent) = self.continent {
            updates.push(format!("continent => {:?}", continent));
        }
        if let Some(ref continent_code) = self.continent_code {
            updates.push(format!("continent_code => {:?}", continent_code));
        }
        if let Some(ref country) = self.country {
            updates.push(format!("country => {:?}", country));
        }
        if let Some(ref country_code) = self.country_code {
            updates.push(format!("country_code => {:?}", country_code));
        }
        if let Some(ref city) = self.city {
            updates.push(format!("city => {:?}", city));
        }
        if let Some(ref latitude) = self.latitude {
            updates.push(format!("latitude => {:?}", latitude));
        }
        if let Some(ref longitude) = self.longitude {
            updates.push(format!("longitude => {:?}", longitude));
        }
        if let Some(ref asn) = self.asn {
            updates.push(format!("asn => {:?}", asn));
        }
        if let Some(ref as_org) = self.as_org {
            updates.push(format!("as_org => {:?}", as_org));
        }

        write!(w, "{}", updates.join(", "))
    }
}

impl Printable<PrintableIpAddr> for NewIpAddrOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableIpAddr> {
        Ok(PrintableIpAddr {
            value: self.value.parse()?,
        })
    }
}
