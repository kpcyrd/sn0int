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

impl Model for Email {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

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

    fn id(&self) -> i32 {
        self.id
    }

    fn value(&self) -> &Self::ID {
        &self.value
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use schema::emails::dsl::*;

        let domain = emails.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use schema::emails::dsl::*;

        let email = emails.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(email)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use schema::emails::dsl::*;

        let email = emails.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(email)
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

impl DisplayableDetailed for DetailedEmail {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.id(w, self.id)?;
        self.green_debug(w, &self.value)?;
        Ok(())
    }

    #[inline]
    fn children(&self, _w: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

display_detailed!(DetailedEmail);

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
    pub valid: Option<bool>,
}

impl<'a> InsertableStruct<Email> for NewEmail<'a> {
    fn value(&self) -> &str {
        self.value
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(emails::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl<'a> Upsertable<Email> for NewEmail<'a> {
    type Update = EmailUpdate;

    fn upsert(self, existing: &Email) -> Self::Update {
        Self::Update {
            id: existing.id,
            valid: Self::upsert_opt(self.valid, &existing.valid),
        }
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="emails"]
pub struct NewEmailOwned {
    pub value: String,
    pub valid: Option<bool>,
}

impl Printable<PrintableEmail> for NewEmailOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableEmail> {
        Ok(PrintableEmail {
            value: self.value.to_string(),
        })
    }
}

pub type InsertEmail = NewEmailOwned;

impl LuaInsertToNewOwned for InsertEmail {
    type Target = NewEmailOwned;

    fn try_into_new(self) -> Result<NewEmailOwned> {
        Ok(self)
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="emails"]
pub struct EmailUpdate {
    pub id: i32,
    pub valid: Option<bool>,
}

impl Upsert for EmailUpdate {
    fn is_dirty(&self) -> bool {
        self.valid.is_some()
    }

    fn generic(self) -> Update {
        Update::Email(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_email(self)
    }
}

impl Updateable<Email> for EmailUpdate {
    fn changeset(&mut self, existing: &Email) {
        Self::clear_if_equal(&mut self.valid, &existing.valid);
    }

    fn fmt(&self, updates: &mut Vec<String>) {
        Self::push_value(updates, "valid", &self.valid);
    }
}
