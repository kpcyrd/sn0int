use crate::errors::*;
//use crate::fmt::Write;
use crate::fmt::colors::*;
use diesel;
use diesel::prelude::*;
use crate::models::*;


#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="breaches"]
pub struct Breach {
    pub id: i32,
    pub value: String,
    pub unscoped: bool,
}

impl Model for Breach {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::breaches::dsl::*;

        let results = breaches.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::breaches::dsl::*;

        let query = breaches.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::breaches::dsl::*;

        diesel::delete(breaches.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::breaches::dsl::*;

        diesel::delete(breaches.filter(id.eq(my_id)))
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
        use crate::schema::breaches::dsl::*;

        let domain = breaches.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::breaches::dsl::*;

        let breach = breaches.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(breach)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::breaches::dsl::*;

        let breach = breaches.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(breach)
    }
}

impl Scopable for Breach {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::breaches::dsl::*;

        diesel::update(breaches.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::breaches::dsl::*;

        diesel::update(breaches.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

impl Breach {
    fn emails(&self, db: &Database) -> Result<Vec<Email>> {
        let email_ids = BreachEmail::belonging_to(self).select(breach_emails::email_id);
        emails::table
            .filter(emails::id.eq_any(email_ids))
            .load::<Email>(db.db())
            .map_err(Error::from)
    }
}

pub struct PrintableBreach {
    value: String,
}

impl fmt::Display for PrintableBreach {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintableBreach> for Breach {
    fn printable(&self, _db: &Database) -> Result<PrintableBreach> {
        Ok(PrintableBreach {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedBreach {
    id: i32,
    value: String,
    emails: Vec<PrintableEmail>,
    unscoped: bool,
}

impl DisplayableDetailed for DetailedBreach {
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
        for email in &self.emails {
            w.child(email)?;
        }
        Ok(())
    }
}

display_detailed!(DetailedBreach);

impl Detailed for Breach {
    type T = DetailedBreach;

    fn detailed(&self, db: &Database) -> Result<Self::T> {
        let emails = self.emails(db)?.into_iter()
            .map(|sd| sd.printable(db))
            .collect::<Result<_>>()?;

        Ok(DetailedBreach {
            id: self.id,
            value: self.value.to_string(),
            emails,
            unscoped: self.unscoped,
        })
    }
}

#[derive(Insertable)]
#[table_name="breaches"]
pub struct NewBreach<'a> {
    pub value: &'a str,
}

impl<'a> InsertableStruct<Breach> for NewBreach<'a> {
    fn value(&self) -> &str {
        self.value
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(breaches::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl<'a> Upsertable<Breach> for NewBreach<'a> {
    type Update = NullUpdate;

    fn upsert(self, existing: &Breach) -> Self::Update {
        Self::Update {
            id: existing.id,
        }
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="breaches"]
pub struct NewBreachOwned {
    pub value: String,
}

impl Printable<PrintableBreach> for NewBreachOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableBreach> {
        Ok(PrintableBreach {
            value: self.value.to_string(),
        })
    }
}

pub type InsertBreach = NewBreachOwned;

impl LuaInsertToNewOwned for InsertBreach {
    type Target = NewBreachOwned;

    fn try_into_new(self) -> Result<NewBreachOwned> {
        Ok(self)
    }
}
