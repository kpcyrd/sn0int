use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::fmt::colors::*;
use diesel::prelude::*;
use crate::models::*;

#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="domains"]
pub struct Domain {
    pub id: i32,
    pub value: String,
    pub unscoped: bool,
}

impl Model for Domain {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::domains::dsl::*;

        let results = domains.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::domains::dsl::*;

        let query = domains.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::domains::dsl::*;

        diesel::delete(domains.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::domains::dsl::*;

        diesel::delete(domains.filter(id.eq(my_id)))
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
        use crate::schema::domains::dsl::*;

        let domain = domains.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::domains::dsl::*;

        let domain = domains.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::domains::dsl::*;

        let domain = domains.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(domain)
    }
}

impl Scopable for Domain {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn set_scoped(&self, db: &Database, my_value: bool) -> Result<()> {
        use crate::schema::domains::dsl::*;
        diesel::update(domains.filter(id.eq(self.id)))
            .set(unscoped.eq(!my_value))
            .execute(db.db())?;
        Ok(())
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::domains::dsl::*;

        diesel::update(domains.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::domains::dsl::*;

        diesel::update(domains.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

impl Domain {
    fn subdomains(&self, db: &Database) -> Result<Vec<Subdomain>> {
        Subdomain::belonging_to(self)
            .load(db.db())
            .map_err(Error::from)
    }
}

pub struct DetailedDomain {
    id: i32,
    value: String,
    subdomains: Vec<PrintableSubdomain>,
    unscoped: bool,
}

impl DisplayableDetailed for DetailedDomain {
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
        for subdomain in &self.subdomains {
            w.child(subdomain)?;
        }
        Ok(())
    }
}

display_detailed!(DetailedDomain);

impl Detailed for Domain {
    type T = DetailedDomain;

    fn detailed(&self, db: &Database) -> Result<Self::T> {
        let subdomains = self.subdomains(db)?.into_iter()
            .map(|sd| sd.printable(db))
            .collect::<Result<_>>()?;

        Ok(DetailedDomain {
            id: self.id,
            value: self.value.to_string(),
            subdomains,
            unscoped: self.unscoped,
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="domains"]
pub struct NewDomain {
    pub value: String,
    pub unscoped: bool,
}

impl InsertableStruct<Domain> for NewDomain {
    fn value(&self) -> &str {
        &self.value
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.unscoped = !scoped;
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(domains::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl Upsertable<Domain> for NewDomain {
    type Update = NullUpdate;

    fn upsert(self, existing: &Domain) -> Self::Update {
        Self::Update {
            id: existing.id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertDomain {
    pub value: String,
}

impl InsertToNew for InsertDomain {
    type Target = NewDomain;

    fn try_into_new(self) -> Result<NewDomain> {
        let value = self.value.to_lowercase();
        if value.contains('*') {
            bail!("Asterisks inside domains are not valid");
        }
        Ok(NewDomain {
            value,
            unscoped: false,
        })
    }
}
