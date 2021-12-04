use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::fmt::colors::*;
use diesel::prelude::*;
use crate::models::*;
use chrono::NaiveDateTime;

#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="cryptoaddrs"]
pub struct CryptoAddr {
    pub id: i32,
    pub value: String,
    pub currency: Option<String>,
    pub denominator: Option<i32>,
    pub balance: Option<i64>,
    pub received: Option<i64>,
    pub first_seen: Option<NaiveDateTime>,
    pub last_withdrawal: Option<NaiveDateTime>,
    pub unscoped: bool,
    pub description: Option<String>,
}

impl Model for CryptoAddr {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::cryptoaddrs::dsl::*;

        let results = cryptoaddrs.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::cryptoaddrs::dsl::*;

        let query = cryptoaddrs.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter_with_param(db: &Database, filter: &Filter, param: &str) -> Result<Vec<Self>> {
        use crate::schema::cryptoaddrs::dsl::*;

        let query = cryptoaddrs
            .filter(currency.eq(param))
            .filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::cryptoaddrs::dsl::*;

        diesel::delete(cryptoaddrs.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::cryptoaddrs::dsl::*;

        diesel::delete(cryptoaddrs.filter(id.eq(my_id)))
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
        use crate::schema::cryptoaddrs::dsl::*;

        let domain = cryptoaddrs.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::cryptoaddrs::dsl::*;

        let cryptoaddr = cryptoaddrs.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(cryptoaddr)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::cryptoaddrs::dsl::*;

        let cryptoaddr = cryptoaddrs.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(cryptoaddr)
    }
}

impl Scopable for CryptoAddr {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn set_scoped(&self, db: &Database, my_value: bool) -> Result<()> {
        use crate::schema::cryptoaddrs::dsl::*;
        diesel::update(cryptoaddrs.filter(id.eq(self.id)))
            .set(unscoped.eq(!my_value))
            .execute(db.db())?;
        Ok(())
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::cryptoaddrs::dsl::*;

        diesel::update(cryptoaddrs.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::cryptoaddrs::dsl::*;

        diesel::update(cryptoaddrs.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

pub struct PrintableCryptoAddr {
    value: String,
}

impl fmt::Display for PrintableCryptoAddr {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintableCryptoAddr> for CryptoAddr {
    fn printable(&self, _db: &Database) -> Result<PrintableCryptoAddr> {
        Ok(PrintableCryptoAddr {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedCryptoAddr {
    id: i32,
    value: String,
    currency: Option<String>,
    denominator: Option<i32>,
    balance: Option<i64>,
    received: Option<i64>,
    first_seen: Option<NaiveDateTime>,
    last_withdrawal: Option<NaiveDateTime>,
    unscoped: bool,
    description: Option<String>,
}

#[inline]
fn add_currency(w: &mut fmt::DetailFormatter, label: &str, num: &Option<i64>, denominator: &Option<i32>) -> fmt::Result {
    if let Some(&num) = num.as_ref() {
        let denominator = denominator.unwrap_or(0);
        let display = display_currency(num as u64, denominator as usize);
        if num > 0 {
            w.display_label::<Green, _>(label, display)?;
        } else {
            w.display_label::<Red, _>(label, display)?;
        }
    }
    Ok(())
}

impl DisplayableDetailed for DetailedCryptoAddr {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        w.id(self.id)?;
        w.debug::<Green, _>(&self.value)?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.currency)?;
        add_currency(w, "balance", &self.balance, &self.denominator)?;
        add_currency(w, "received", &self.received, &self.denominator)?;
        w.end_group()?;

        w.start_group();
        w.opt_debug_label::<Yellow, _>("first_seen", &self.first_seen)?;
        w.opt_debug_label::<Yellow, _>("last_withdrawal", &self.last_withdrawal)?;
        w.opt_debug_label::<Yellow, _>("description", &self.description)?;
        w.end_group()?;

        Ok(())
    }

    #[inline]
    fn children(&self, _w: &mut fmt::DetailFormatter) -> fmt::Result {
        Ok(())
    }
}

display_detailed!(DetailedCryptoAddr);

impl Detailed for CryptoAddr {
    type T = DetailedCryptoAddr;

    fn detailed(&self, _db: &Database) -> Result<Self::T> {
        Ok(DetailedCryptoAddr {
            id: self.id,
            value: self.value.to_string(),
            currency: self.currency.clone(),
            denominator: self.denominator,
            balance: self.balance,
            received: self.received,
            first_seen: self.first_seen,
            last_withdrawal: self.last_withdrawal,
            unscoped: self.unscoped,
            description: self.description.clone(),
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="cryptoaddrs"]
pub struct NewCryptoAddr {
    pub value: String,
    pub currency: Option<String>,
    pub denominator: Option<i32>,
    pub balance: Option<i64>,
    pub received: Option<i64>,
    pub first_seen: Option<NaiveDateTime>,
    pub last_withdrawal: Option<NaiveDateTime>,
    pub unscoped: bool,
    pub description: Option<String>,
}

impl InsertableStruct<CryptoAddr> for NewCryptoAddr {
    fn value(&self) -> &str {
        &self.value
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.unscoped = !scoped;
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(cryptoaddrs::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl Upsertable<CryptoAddr> for NewCryptoAddr {
    type Update = CryptoAddrUpdate;

    fn upsert(self, existing: &CryptoAddr) -> Self::Update {
        Self::Update {
            id: existing.id,
            currency: Self::upsert_opt(self.currency, &existing.currency),
            denominator: Self::upsert_opt(self.denominator, &existing.denominator),
            balance: Self::upsert_opt(self.balance, &existing.balance),
            received: Self::upsert_opt(self.received, &existing.received),
            first_seen: Self::upsert_opt(self.first_seen, &existing.first_seen),
            last_withdrawal: Self::upsert_opt(self.last_withdrawal, &existing.last_withdrawal),
            description: Self::upsert_opt(self.description, &existing.description),
        }
    }
}

impl Printable<PrintableCryptoAddr> for NewCryptoAddr {
    fn printable(&self, _db: &Database) -> Result<PrintableCryptoAddr> {
        Ok(PrintableCryptoAddr {
            value: self.value.to_string(),
        })
    }
}

pub type InsertCryptoAddr = NewCryptoAddr;

impl InsertToNew for InsertCryptoAddr {
    type Target = NewCryptoAddr;

    #[inline]
    fn try_into_new(self) -> Result<NewCryptoAddr> {
        Ok(self)
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="cryptoaddrs"]
pub struct CryptoAddrUpdate {
    pub id: i32,
    pub currency: Option<String>,
    pub denominator: Option<i32>,
    pub balance: Option<i64>,
    pub received: Option<i64>,
    pub first_seen: Option<NaiveDateTime>,
    pub last_withdrawal: Option<NaiveDateTime>,
    pub description: Option<String>,
}

impl Upsert for CryptoAddrUpdate {
    fn is_dirty(&self) -> bool {
        self.currency.is_some() ||
        self.denominator.is_some() ||
        self.balance.is_some() ||
        self.received.is_some() ||
        self.first_seen.is_some() ||
        self.last_withdrawal.is_some() ||
        self.description.is_some()
    }

    fn generic(self) -> Update {
        Update::CryptoAddr(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_cryptoaddr(self)
    }
}

impl Updateable<CryptoAddr> for CryptoAddrUpdate {
    fn changeset(&mut self, existing: &CryptoAddr) {
        Self::clear_if_equal(&mut self.currency, &existing.currency);
        Self::clear_if_equal(&mut self.denominator, &existing.denominator);
        Self::clear_if_equal(&mut self.balance, &existing.balance);
        Self::clear_if_equal(&mut self.received, &existing.received);
        Self::clear_if_greater_or_equal(&mut self.first_seen, &existing.first_seen);
        Self::clear_if_lower_or_equal(&mut self.last_withdrawal, &existing.last_withdrawal);
        Self::clear_if_equal(&mut self.description, &existing.description);
    }

    fn fmt(&self, updates: &mut Vec<String>, colors: bool) {
        Self::push_value(updates, "currency", &self.currency, colors);
        Self::push_value(updates, "denominator", &self.denominator, colors);
        Self::push_value(updates, "balance", &self.balance, colors);
        Self::push_value(updates, "received", &self.received, colors);
        Self::push_value(updates, "first_seen", &self.first_seen, colors);
        Self::push_value(updates, "last_withdrawal", &self.last_withdrawal, colors);
        Self::push_value(updates, "description", &self.description, colors);
    }
}

// ensure precision by working around floats
fn display_currency(value: u64, denominator: usize) -> String {
    let x = format!("{:>0width$}", value, width=denominator+1);
    let (a, b) = x.split_at(x.len()-denominator);
    let mut x = format!("{}.{}", a, b).chars().collect::<Vec<_>>();

    loop {
        match x.last() {
            Some('0') => x.pop(),
            Some('.') => {
                x.pop();
                break;
            },
            _ => break,
        };
    }

    x.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_1() {
        let x = display_currency(0, 8);
        assert_eq!(x, "0");
    }

    #[test]
    fn test_bitcoin_2() {
        let x = display_currency(1, 8);
        assert_eq!(x, "0.00000001");
    }

    #[test]
    fn test_bitcoin_3() {
        let x = display_currency(100000000, 8);
        assert_eq!(x, "1");
    }

    #[test]
    fn test_bitcoin_4() {
        let x = display_currency(10000000000, 8);
        assert_eq!(x, "100");
    }

    #[test]
    fn test_bitcoin_5() {
        let x = display_currency(123450000, 8);
        assert_eq!(x, "1.2345");
    }

    #[test]
    fn test_bitcoin_6() {
        let x = display_currency(12345, 8);
        assert_eq!(x, "0.00012345");
    }

    #[test]
    fn test_zero_1() {
        let x = display_currency(0, 8);
        assert_eq!(x, "0");
    }

    #[test]
    fn test_zero_2() {
        let x = display_currency(12345, 0);
        assert_eq!(x, "12345");
    }
}
