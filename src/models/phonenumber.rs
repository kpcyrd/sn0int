use crate::errors::*;
use diesel;
use diesel::prelude::*;
use crate::models::*;


#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="phonenumbers"]
pub struct PhoneNumber {
    pub id: i32,
    pub value: String,
    pub name: Option<String>,
    pub unscoped: bool,
    pub valid: Option<bool>,
}

impl Model for PhoneNumber {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::phonenumbers::dsl::*;

        let results = phonenumbers.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::phonenumbers::dsl::*;

        let query = phonenumbers.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::phonenumbers::dsl::*;

        diesel::delete(phonenumbers.filter(filter.sql()))
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
        use crate::schema::phonenumbers::dsl::*;

        let domain = phonenumbers.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::phonenumbers::dsl::*;

        let email = phonenumbers.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(email)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::phonenumbers::dsl::*;

        let email = phonenumbers.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(email)
    }
}

impl Scopable for PhoneNumber {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::phonenumbers::dsl::*;

        diesel::update(phonenumbers.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::phonenumbers::dsl::*;

        diesel::update(phonenumbers.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

pub struct PrintablePhoneNumber {
    value: String,
}

impl fmt::Display for PrintablePhoneNumber {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintablePhoneNumber> for PhoneNumber {
    fn printable(&self, _db: &Database) -> Result<PrintablePhoneNumber> {
        Ok(PrintablePhoneNumber {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedPhoneNumber {
    id: i32,
    value: String,
    name: Option<String>,
    unscoped: bool,
    valid: Option<bool>,
}

impl DisplayableDetailed for DetailedPhoneNumber {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.id(w, self.id)?;
        self.green_debug(w, &self.value)?;

        match (&self.name, self.valid) {
            (None, None) => (),
            (name, valid) => {
                write!(w, " [")?;
                let mut need_separator = false;

                if let Some(name) = name {
                    self.yellow_debug(w, name)?;
                    need_separator = true;
                }

                if let Some(valid) = valid {
                    if need_separator {
                        write!(w, ", ")?;
                    }

                    if valid {
                        self.green_display(w, "valid")?;
                    } else {
                        self.red_display(w, "invalid")?;
                    }
                }
                write!(w, "]")?;
            },
        }

        Ok(())
    }

    #[inline]
    fn children(&self, _w: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

display_detailed!(DetailedPhoneNumber);

impl Detailed for PhoneNumber {
    type T = DetailedPhoneNumber;

    fn detailed(&self, _db: &Database) -> Result<Self::T> {
        Ok(DetailedPhoneNumber {
            id: self.id,
            value: self.value.to_string(),
            name: self.name.clone(),
            unscoped: self.unscoped,
            valid: self.valid,
        })
    }
}

#[derive(Insertable)]
#[table_name="phonenumbers"]
pub struct NewPhoneNumber<'a> {
    pub value: &'a str,
    pub name: Option<&'a String>,
    pub valid: Option<bool>,
}

impl<'a> InsertableStruct<PhoneNumber> for NewPhoneNumber<'a> {
    fn value(&self) -> &str {
        self.value
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(phonenumbers::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl<'a> Upsertable<PhoneNumber> for NewPhoneNumber<'a> {
    type Update = PhoneNumberUpdate;

    fn upsert(self, existing: &PhoneNumber) -> Self::Update {
        Self::Update {
            id: existing.id,
            name: Self::upsert_str(self.name, &existing.name),
            valid: Self::upsert_opt(self.valid, &existing.valid),
        }
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="phonenumbers"]
pub struct NewPhoneNumberOwned {
    pub value: String,
    pub name: Option<String>,
    pub valid: Option<bool>,
}

impl Printable<PrintablePhoneNumber> for NewPhoneNumberOwned {
    fn printable(&self, _db: &Database) -> Result<PrintablePhoneNumber> {
        Ok(PrintablePhoneNumber {
            value: self.value.to_string(),
        })
    }
}

pub type InsertPhoneNumber = NewPhoneNumberOwned;

// TODO: enforce valid E.164 number?
impl LuaInsertToNewOwned for InsertPhoneNumber {
    type Target = NewPhoneNumberOwned;

    fn try_into_new(self) -> Result<NewPhoneNumberOwned> {
        if self.value.starts_with('+') {
            bail!("E.164 phone number must start with '+'");
        }

        if !self.value[1..].chars().all(char::is_numeric) {
            bail!("E.164 phone number must only contain numbers");
        }

        Ok(self)
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="phonenumbers"]
pub struct PhoneNumberUpdate {
    pub id: i32,
    pub name: Option<String>,
    pub valid: Option<bool>,
}

impl Upsert for PhoneNumberUpdate {
    fn is_dirty(&self) -> bool {
        self.name.is_some() ||
            self.valid.is_some()
    }

    fn generic(self) -> Update {
        Update::PhoneNumber(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_phonenumber(self)
    }
}

impl Updateable<PhoneNumber> for PhoneNumberUpdate {
    fn changeset(&mut self, existing: &PhoneNumber) {
        Self::clear_if_equal(&mut self.name, &existing.name);
        Self::clear_if_equal(&mut self.valid, &existing.valid);
    }

    fn fmt(&self, updates: &mut Vec<String>) {
        Self::push_value(updates, "name", &self.name);
        Self::push_value(updates, "valid", &self.valid);
    }
}
