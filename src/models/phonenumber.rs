use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::fmt::colors::*;
use crate::models::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="phonenumbers"]
pub struct PhoneNumber {
    pub id: i32,
    pub value: String,
    pub name: Option<String>,
    pub unscoped: bool,
    pub valid: Option<bool>,
    pub last_online: Option<NaiveDateTime>,
    pub country: Option<String>,
    pub carrier: Option<String>,
    pub line: Option<String>,
    pub is_ported: Option<bool>,
    pub last_ported: Option<NaiveDateTime>,
    pub caller_name: Option<String>,
    pub caller_type: Option<String>,
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

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::phonenumbers::dsl::*;

        diesel::delete(phonenumbers.filter(id.eq(my_id)))
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

        let phonenumber = phonenumbers.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(phonenumber)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::phonenumbers::dsl::*;

        let phonenumber = phonenumbers.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(phonenumber)
    }
}

impl Scopable for PhoneNumber {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn set_scoped(&self, db: &Database, my_value: bool) -> Result<()> {
        use crate::schema::phonenumbers::dsl::*;
        diesel::update(phonenumbers.filter(id.eq(self.id)))
            .set(unscoped.eq(!my_value))
            .execute(db.db())?;
        Ok(())
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
    country: Option<String>,
    carrier: Option<String>,
    line: Option<String>,
    caller_name: Option<String>,
    caller_type: Option<String>,
}

impl DisplayableDetailed for DetailedPhoneNumber {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        w.id(self.id)?;
        w.debug::<Green, _>(&self.value)?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.name)?;

        if let Some(valid) = &self.valid {
            if *valid {
                w.display::<Green, _>("valid")?;
            } else {
                w.display::<Red, _>("invalid")?;
            }
        }

        w.opt_debug::<Yellow, _>(&self.country)?;
        w.opt_debug::<Yellow, _>(&self.carrier)?;
        w.opt_debug::<Yellow, _>(&self.line)?;
        w.opt_debug::<Yellow, _>(&self.caller_name)?;
        w.opt_debug::<Yellow, _>(&self.caller_type)?;
        w.end_group()?;

        Ok(())
    }

    #[inline]
    fn children(&self, _w: &mut fmt::DetailFormatter) -> fmt::Result {
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
            country: self.country.clone(),
            carrier: self.carrier.clone(),
            line: self.line.clone(),
            caller_name: self.caller_name.clone(),
            caller_type: self.caller_type.clone(),
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="phonenumbers"]
pub struct NewPhoneNumber {
    pub value: String,
    pub name: Option<String>,
    pub valid: Option<bool>,
    pub last_online: Option<NaiveDateTime>,
    pub country: Option<String>,
    pub carrier: Option<String>,
    pub line: Option<String>,
    pub is_ported: Option<bool>,
    pub last_ported: Option<NaiveDateTime>,
    pub caller_name: Option<String>,
    pub caller_type: Option<String>,

    pub unscoped: bool,
}

impl InsertableStruct<PhoneNumber> for NewPhoneNumber {
    fn value(&self) -> &str {
        &self.value
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.unscoped = !scoped;
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(phonenumbers::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl Upsertable<PhoneNumber> for NewPhoneNumber {
    type Update = PhoneNumberUpdate;

    fn upsert(self, existing: &PhoneNumber) -> Self::Update {
        Self::Update {
            id: existing.id,
            name: Self::upsert_opt(self.name, &existing.name),
            valid: Self::upsert_opt(self.valid, &existing.valid),
            last_online: Self::upsert_opt(self.last_online, &existing.last_online),
            country: Self::upsert_opt(self.country, &existing.country),
            carrier: Self::upsert_opt(self.carrier, &existing.carrier),
            line: Self::upsert_opt(self.line, &existing.line),
            is_ported: Self::upsert_opt(self.is_ported, &existing.is_ported),
            last_ported: Self::upsert_opt(self.last_ported, &existing.last_ported),
            caller_name: Self::upsert_opt(self.caller_name, &existing.caller_name),
            caller_type: Self::upsert_opt(self.caller_type, &existing.caller_type),
        }
    }
}

impl Printable<PrintablePhoneNumber> for NewPhoneNumber {
    fn printable(&self, _db: &Database) -> Result<PrintablePhoneNumber> {
        Ok(PrintablePhoneNumber {
            value: self.value.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertPhoneNumber {
    pub value: String,
    pub name: Option<String>,
    pub valid: Option<bool>,
    pub last_online: Option<NaiveDateTime>,
    pub country: Option<String>,
    pub carrier: Option<String>,
    pub line: Option<String>,
    pub is_ported: Option<bool>,
    pub last_ported: Option<NaiveDateTime>,
    pub caller_name: Option<String>,
    pub caller_type: Option<String>,
}

// TODO: enforce valid E.164 number?
impl InsertToNew for InsertPhoneNumber {
    type Target = NewPhoneNumber;

    fn try_into_new(self) -> Result<NewPhoneNumber> {
        if !self.value.starts_with('+') {
            bail!("E.164 phone number must start with '+'");
        }

        if !self.value[1..].chars().all(char::is_numeric) {
            bail!("E.164 phone number must only contain numbers");
        }

        Ok(NewPhoneNumber {
            value: self.value,
            name: self.name,
            valid: self.valid,
            last_online: self.last_online,
            country: self.country,
            carrier: self.carrier,
            line: self.line,
            is_ported: self.is_ported,
            last_ported: self.last_ported,
            caller_name: self.caller_name,
            caller_type: self.caller_type,

            unscoped: false,
        })
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="phonenumbers"]
pub struct PhoneNumberUpdate {
    pub id: i32,
    pub name: Option<String>,
    pub valid: Option<bool>,
    pub last_online: Option<NaiveDateTime>,
    pub country: Option<String>,
    pub carrier: Option<String>,
    pub line: Option<String>,
    pub is_ported: Option<bool>,
    pub last_ported: Option<NaiveDateTime>,
    pub caller_name: Option<String>,
    pub caller_type: Option<String>,
}

impl Upsert for PhoneNumberUpdate {
    fn is_dirty(&self) -> bool {
        self.name.is_some() ||
        self.valid.is_some() ||
        self.last_online.is_some() ||
        self.country.is_some() ||
        self.carrier.is_some() ||
        self.line.is_some() ||
        self.is_ported.is_some() ||
        self.last_ported.is_some() ||
        self.caller_name.is_some() ||
        self.caller_type.is_some()
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
        Self::clear_if_equal(&mut self.last_online, &existing.last_online);
        Self::clear_if_equal(&mut self.country, &existing.country);
        Self::clear_if_equal(&mut self.carrier, &existing.carrier);
        Self::clear_if_equal(&mut self.line, &existing.line);
        Self::clear_if_equal(&mut self.is_ported, &existing.is_ported);
        Self::clear_if_equal(&mut self.last_ported, &existing.last_ported);
        Self::clear_if_equal(&mut self.caller_name, &existing.caller_name);
        Self::clear_if_equal(&mut self.caller_type, &existing.caller_type);
    }

    fn fmt(&self, updates: &mut Vec<String>, colors: bool) {
        Self::push_value(updates, "name", &self.name, colors);
        Self::push_value(updates, "valid", &self.valid, colors);
        Self::push_value(updates, "last_online", &self.last_online, colors);
        Self::push_value(updates, "country", &self.country, colors);
        Self::push_value(updates, "carrier", &self.carrier, colors);
        Self::push_value(updates, "line", &self.line, colors);
        Self::push_value(updates, "is_ported", &self.is_ported, colors);
        Self::push_value(updates, "last_ported", &self.last_ported, colors);
        Self::push_value(updates, "caller_name", &self.caller_name, colors);
        Self::push_value(updates, "caller_type", &self.caller_type, colors);
    }
}
