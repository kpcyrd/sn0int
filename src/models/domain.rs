use errors::*;
use diesel::prelude::*;
use models::*;


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

#[derive(Insertable)]
#[table_name="domains"]
pub struct NewDomain<'a> {
    pub value: &'a str,
}
