use errors::*;
use diesel::prelude::*;
use json::LuaJsonValue;
use models::*;
use serde_json;


#[derive(Identifiable, Queryable, Serialize, PartialEq, Debug)]
#[table_name="domains"]
pub struct Domain {
    pub id: i32,
    pub value: String,
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Model for Domain {
    type ID = str;

    fn list(db: &Database) -> Result<Vec<Self>> {
        use schema::domains::dsl::*;

        let results = domains.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use schema::domains::dsl::*;

        let query = domains.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use schema::domains::dsl::*;

        let domain = domains.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn id(db: &Database, query: &Self::ID) -> Result<i32> {
        use schema::domains::dsl::*;

        let domain_id = domains.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())?;

        Ok(domain_id)
    }

    fn id_opt(db: &Database, query: &Self::ID) -> Result<Option<i32>> {
        use schema::domains::dsl::*;

        let domain_id = domains.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())
            .optional()?;

        Ok(domain_id)
    }
}

pub struct PrintableDomain {
    value: String,
}

impl fmt::Display for PrintableDomain {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintableDomain> for Domain {
    fn printable(&self, _db: &Database) -> Result<PrintableDomain> {
        Ok(PrintableDomain {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedDomain {
    id: i32,
    value: String,
}

impl fmt::Display for DetailedDomain {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "\x1b[32m#{}\x1b[0m, \x1b[32m{:?}\x1b[0m", self.id, self.value)
    }
}

impl Detailed for Domain {
    type T = DetailedDomain;

    fn detailed(&self, _db: &Database) -> Result<Self::T> {
        Ok(DetailedDomain {
            id: self.id,
            value: self.value.to_string(),
        })
    }
}

#[derive(Insertable)]
#[table_name="domains"]
pub struct NewDomain<'a> {
    pub value: &'a str,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="domains"]
pub struct NewDomainOwned {
    pub value: String,
}

impl NewDomainOwned {
    pub fn from_lua(x: LuaJsonValue) -> Result<NewDomainOwned> {
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

impl Printable<PrintableDomain> for NewDomainOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableDomain> {
        Ok(PrintableDomain {
            value: self.value.to_string(),
        })
    }
}
