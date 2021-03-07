use crate::errors::*;
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use crate::models::*;

#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[belongs_to(Breach)]
#[belongs_to(Email)]
#[table_name="breach_emails"]
pub struct BreachEmail {
    pub id: i32,
    pub breach_id: i32,
    pub email_id: i32,
    pub password: Option<String>,
}

impl Model for BreachEmail {
    type ID = (i32, i32, Option<String>);

    fn to_string(&self) -> String {
        unimplemented!("BreachEmail can not be printed")
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::breach_emails::dsl::*;

        let results = breach_emails.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::breach_emails::dsl::*;

        let query = breach_emails.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::breach_emails::dsl::*;

        diesel::delete(breach_emails.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::breach_emails::dsl::*;

        diesel::delete(breach_emails.filter(id.eq(my_id)))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn id(&self) -> i32 {
        self.id
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use crate::schema::breach_emails::dsl::*;

        let breach_email = breach_emails.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(breach_email)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::breach_emails::dsl::*;

        let (my_breach_id, my_email_id, my_password) = query;

        let query = breach_emails.filter(breach_id.eq(my_breach_id))
                                 .filter(email_id.eq(my_email_id));
        let breach_email = if let Some(my_password) = my_password {
           query
               .filter(password.is_null().or(password.eq(my_password)))
               .first::<Self>(db.db())?
        } else {
           query
               .first::<Self>(db.db())?
        };

        Ok(breach_email)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::breach_emails::dsl::*;

        let (my_breach_id, my_email_id, my_password) = query;

        let query = breach_emails.filter(breach_id.eq(my_breach_id))
                                 .filter(email_id.eq(my_email_id));
        let breach_email = if let Some(my_password) = my_password {
           query
               .filter(password.is_null().or(password.eq(my_password)))
               .first::<Self>(db.db())
               .optional()?
        } else {
           query
               .first::<Self>(db.db())
               .optional()?
        };

        Ok(breach_email)
    }
}

impl BreachEmail {
    pub fn breach(&self, db: &Database) -> Result<Breach> {
        Breach::by_id(db, self.breach_id)
    }
}

pub struct PrintableBreachEmail {
    breach: String,
    email: String,
    password: Option<String>,
}

impl fmt::Display for PrintableBreachEmail {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?} -> {:?}", self.breach, self.email)?;
        if let Some(password) = &self.password {
            write!(w, " ({:?})", password)?;
        }
        Ok(())
    }
}

impl Printable<PrintableBreachEmail> for BreachEmail {
    fn printable(&self, db: &Database) -> Result<PrintableBreachEmail> {
        let breach = Breach::by_id(db, self.breach_id)?;
        let email = Email::by_id(db, self.email_id)?;
        Ok(PrintableBreachEmail {
            breach: breach.value,
            email: email.value,
            password: self.password.clone(),
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="breach_emails"]
pub struct NewBreachEmail {
    pub breach_id: i32,
    pub email_id: i32,
    pub password: Option<String>,
}

impl Upsertable<BreachEmail> for NewBreachEmail {
    type Update = BreachEmailUpdate;

    fn upsert(self, existing: &BreachEmail) -> Self::Update {
        Self::Update {
            id: existing.id,
            password: Self::upsert_opt(self.password, &existing.password),
        }
    }
}

impl Printable<PrintableBreachEmail> for NewBreachEmail {
    fn printable(&self, db: &Database) -> Result<PrintableBreachEmail> {
        let breach = Breach::by_id(db, self.breach_id)?;
        let email = Email::by_id(db, self.email_id)?;
        Ok(PrintableBreachEmail {
            breach: breach.value,
            email: email.value,
            password: self.password.clone(),
        })
    }
}

pub type InsertBreachEmail = NewBreachEmail;

impl InsertToNew for InsertBreachEmail {
    type Target = NewBreachEmail;

    #[inline]
    fn try_into_new(self) -> Result<NewBreachEmail> {
        Ok(self)
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="breach_emails"]
pub struct BreachEmailUpdate {
    pub id: i32,
    pub password: Option<String>,
}

impl Upsert for BreachEmailUpdate {
    fn is_dirty(&self) -> bool {
        self.password.is_some()
    }

    fn generic(self) -> Update {
        Update::BreachEmail(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_breach_email(self)
    }
}

impl Updateable<BreachEmail> for BreachEmailUpdate {
    fn changeset(&mut self, existing: &BreachEmail) {
        Self::clear_if_equal(&mut self.password, &existing.password);
    }

    fn fmt(&self, updates: &mut Vec<String>, colors: bool) {
        Self::push_value(updates, "password", &self.password, colors);
    }
}
