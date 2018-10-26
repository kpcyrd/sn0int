use errors::*;
use diesel;
use diesel::prelude::*;
use models::*;


#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="emails"]
pub struct Email {
    pub id: i32,
    pub value: String,
    pub unscoped: bool,
    pub valid: Option<bool>,
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Model for Email {
    type ID = str;

    fn list(db: &Database) -> Result<Vec<Self>> {
        use schema::emails::dsl::*;

        let results = emails.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use schema::emails::dsl::*;

        let query = emails.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::emails::dsl::*;

        diesel::delete(emails.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use schema::emails::dsl::*;

        let domain = emails.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn id(db: &Database, query: &Self::ID) -> Result<i32> {
        use schema::emails::dsl::*;

        let domain_id = emails.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())?;

        Ok(domain_id)
    }

    fn id_opt(db: &Database, query: &Self::ID) -> Result<Option<i32>> {
        use schema::emails::dsl::*;

        let domain_id = emails.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())
            .optional()?;

        Ok(domain_id)
    }
}

impl Scopable for Email {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::emails::dsl::*;

        diesel::update(emails.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::emails::dsl::*;

        diesel::update(emails.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="emails"]
pub struct EmailUpdate {
    pub id: i32,
    pub valid: Option<bool>,
}

impl fmt::Display for EmailUpdate {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if let Some(valid) = self.valid {
            write!(w, "valid => {:?}", valid)?;
        }
        Ok(())
    }
}

pub struct PrintableEmail {
    value: String,
}

impl fmt::Display for PrintableEmail {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintableEmail> for Email {
    fn printable(&self, _db: &Database) -> Result<PrintableEmail> {
        Ok(PrintableEmail {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedEmail {
    id: i32,
    value: String,
    unscoped: bool,
}

impl fmt::Display for DetailedEmail {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if !self.unscoped {
            write!(w, "\x1b[32m#{}\x1b[0m, \x1b[32m{:?}\x1b[0m", self.id, self.value)
        } else {
            write!(w, "\x1b[90m#{}, {:?}\x1b[0m", self.id, self.value)
        }
    }
}

impl Detailed for Email {
    type T = DetailedEmail;

    fn detailed(&self, _db: &Database) -> Result<Self::T> {
        Ok(DetailedEmail {
            id: self.id,
            value: self.value.to_string(),
            unscoped: self.unscoped,
        })
    }
}

#[derive(Insertable)]
#[table_name="emails"]
pub struct NewEmail<'a> {
    pub value: &'a str,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="emails"]
pub struct NewEmailOwned {
    pub value: String,
}

impl Printable<PrintableEmail> for NewEmailOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableEmail> {
        Ok(PrintableEmail {
            value: self.value.to_string(),
        })
    }
}
