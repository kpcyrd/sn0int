use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::fmt::colors::*;
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

    fn set_scoped(&self, db: &Database, my_value: bool) -> Result<()> {
        use crate::schema::breaches::dsl::*;
        diesel::update(breaches.filter(id.eq(self.id)))
            .set(unscoped.eq(!my_value))
            .execute(db.db())?;
        Ok(())
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
    fn emails(&self, db: &Database) -> Result<Vec<(Email, Option<String>)>> {
        use std::result;

        let email_id_pws = BreachEmail::belonging_to(self)
            .select((breach_emails::email_id, breach_emails::password))
            .load::<(i32, Option<String>)>(db.db())?;

        email_id_pws.into_iter()
            .map(|(email_id, password)| {
                emails::table
                    .filter(emails::id.eq(email_id))
                    .first::<Email>(db.db())
                    .map(|email| (email, password))
            })
            .collect::<result::Result<Vec<_>, _>>()
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

pub struct EmailWithPassword {
    email: PrintableEmail,
    password: Option<String>,
}

impl fmt::Display for EmailWithPassword {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{}", self.email)?;
        if let Some(password) = &self.password {
            write!(w, " ({:?})", password)?;
        }
        Ok(())
    }
}

pub struct DetailedBreach {
    id: i32,
    value: String,
    emails: Vec<EmailWithPassword>,
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
            .map(|(sd, password)| Ok(EmailWithPassword {
                email: sd.printable(db)?,
                password,
            }))
            .collect::<Result<_>>()?;

        Ok(DetailedBreach {
            id: self.id,
            value: self.value.to_string(),
            emails,
            unscoped: self.unscoped,
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="breaches"]
pub struct NewBreach {
    pub value: String,
    pub unscoped: bool,
}

impl InsertableStruct<Breach> for NewBreach {
    fn value(&self) -> &str {
        &self.value
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.unscoped = !scoped;
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(breaches::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl Upsertable<Breach> for NewBreach {
    type Update = NullUpdate;

    fn upsert(self, existing: &Breach) -> Self::Update {
        Self::Update {
            id: existing.id,
        }
    }
}

impl Printable<PrintableBreach> for NewBreach {
    fn printable(&self, _db: &Database) -> Result<PrintableBreach> {
        Ok(PrintableBreach {
            value: self.value.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertBreach {
    pub value: String,
}

impl InsertToNew for InsertBreach {
    type Target = NewBreach;

    fn try_into_new(self) -> Result<NewBreach> {
        Ok(NewBreach {
            value: self.value,

            unscoped: false,
        })
    }
}
