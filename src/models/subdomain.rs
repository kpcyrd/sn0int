use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::fmt::colors::*;
use diesel::prelude::*;
use crate::models::*;
use std::result;

#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(Domain)]
#[table_name="subdomains"]
pub struct Subdomain {
    pub id: i32,
    pub domain_id: i32,
    pub value: String,
    pub unscoped: bool,
    pub resolvable: Option<bool>,
}

impl Model for Subdomain {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::subdomains::dsl::*;

        let results = subdomains.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::subdomains::dsl::*;

        let query = subdomains.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::subdomains::dsl::*;

        diesel::delete(subdomains.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::subdomains::dsl::*;

        diesel::delete(subdomains.filter(id.eq(my_id)))
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
        use crate::schema::subdomains::dsl::*;

        let subdomain = subdomains.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(subdomain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::subdomains::dsl::*;

        let subdomain = subdomains.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(subdomain)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::subdomains::dsl::*;

        let subdomain = subdomains.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(subdomain)
    }
}

impl Scopable for Subdomain {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn set_scoped(&self, db: &Database, my_value: bool) -> Result<()> {
        use crate::schema::subdomains::dsl::*;
        diesel::update(subdomains.filter(id.eq(self.id)))
            .set(unscoped.eq(!my_value))
            .execute(db.db())?;
        Ok(())
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::subdomains::dsl::*;

        diesel::update(subdomains.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::subdomains::dsl::*;

        diesel::update(subdomains.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

impl Subdomain {
    fn ip_addrs(&self, db: &Database) -> Result<Vec<IpAddr>> {
        let ipaddr_ids = SubdomainIpAddr::belonging_to(self)
            .select(subdomain_ipaddrs::ip_addr_id)
            .load::<i32>(db.db())?;

        ipaddr_ids.into_iter()
            .map(|ipaddr_id| ipaddrs::table
                .filter(ipaddrs::id.eq(ipaddr_id))
                .first::<IpAddr>(db.db())
            )
            .collect::<result::Result<_, _>>()
            .map_err(Error::from)
    }
}

pub struct PrintableSubdomain {
    value: String,
}

impl fmt::Display for PrintableSubdomain {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintableSubdomain> for Subdomain {
    fn printable(&self, _db: &Database) -> Result<PrintableSubdomain> {
        Ok(PrintableSubdomain {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedSubdomain {
    id: i32,
    value: String,
    ipaddrs: Vec<PrintableIpAddr>,
    unscoped: bool,
}

// TODO: maybe print urls as well
impl DisplayableDetailed for DetailedSubdomain {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        w.id(self.id)?;
        w.debug::<Green, _>(&self.value)?;
        Ok(())
    }

    #[inline]
    fn children(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        for ipaddr in &self.ipaddrs {
            w.child(ipaddr)?;
        }
        Ok(())
    }
}

display_detailed!(DetailedSubdomain);

impl Detailed for Subdomain {
    type T = DetailedSubdomain;

    fn detailed(&self, db: &Database) -> Result<Self::T> {
        let ipaddrs = self.ip_addrs(db)?.into_iter()
            .map(|ip| ip.printable(db))
            .collect::<Result<_>>()?;

        Ok(DetailedSubdomain {
            id: self.id,
            value: self.value.to_string(),
            ipaddrs,
            unscoped: self.unscoped,
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="subdomains"]
pub struct NewSubdomain {
    pub domain_id: i32,
    pub value: String,
    pub resolvable: Option<bool>,
    pub unscoped: bool,
}

impl InsertableStruct<Subdomain> for NewSubdomain {
    fn value(&self) -> &str {
        &self.value
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.unscoped = !scoped;
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(subdomains::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl Upsertable<Subdomain> for NewSubdomain {
    type Update = SubdomainUpdate;

    fn upsert(self, existing: &Subdomain) -> Self::Update {
        Self::Update {
            id: existing.id,
            resolvable: Self::upsert_opt(self.resolvable, &existing.resolvable),
        }
    }
}

impl Printable<PrintableSubdomain> for NewSubdomain {
    fn printable(&self, _db: &Database) -> Result<PrintableSubdomain> {
        Ok(PrintableSubdomain {
            value: self.value.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertSubdomain {
    pub domain_id: i32,
    pub value: String,
    pub resolvable: Option<bool>,
}

impl InsertToNew for InsertSubdomain {
    type Target = NewSubdomain;

    fn try_into_new(self) -> Result<NewSubdomain> {
        let value = self.value.to_lowercase();
        if value.contains('*') {
            bail!("Asterisks inside domains are not valid");
        }
        Ok(NewSubdomain {
            domain_id: self.domain_id,
            value,
            resolvable: self.resolvable,

            unscoped: false,
        })
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="subdomains"]
pub struct SubdomainUpdate {
    pub id: i32,
    pub resolvable: Option<bool>,
}

impl Upsert for SubdomainUpdate {
    fn is_dirty(&self) -> bool {
        self.resolvable.is_some()
    }

    fn generic(self) -> Update {
        Update::Subdomain(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_subdomain(self)
    }
}

impl Updateable<Subdomain> for SubdomainUpdate {
    fn changeset(&mut self, existing: &Subdomain) {
        Self::clear_if_equal(&mut self.resolvable, &existing.resolvable);
    }

    fn fmt(&self, updates: &mut Vec<String>, colors: bool) {
        Self::push_value(updates, "resolvable", &self.resolvable, colors);
    }
}
