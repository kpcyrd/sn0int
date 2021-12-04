use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::fmt::colors::*;
use diesel::prelude::*;
use crate::models::*;

#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="netblocks"]
pub struct Netblock {
    pub id: i32,
    pub family: String,
    pub value: String,
    pub unscoped: bool,
    pub asn: Option<i32>,
    pub as_org: Option<String>,
    pub description: Option<String>,
}

impl Model for Netblock {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::netblocks::dsl::*;

        let results = netblocks.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::netblocks::dsl::*;

        let query = netblocks.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::netblocks::dsl::*;

        diesel::delete(netblocks.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::netblocks::dsl::*;

        diesel::delete(netblocks.filter(id.eq(my_id)))
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
        use crate::schema::netblocks::dsl::*;

        let domain = netblocks.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::netblocks::dsl::*;

        let netblock = netblocks.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(netblock)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::netblocks::dsl::*;

        let netblock = netblocks.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(netblock)
    }
}

impl Scopable for Netblock {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn set_scoped(&self, db: &Database, my_value: bool) -> Result<()> {
        use crate::schema::netblocks::dsl::*;
        diesel::update(netblocks.filter(id.eq(self.id)))
            .set(unscoped.eq(!my_value))
            .execute(db.db())?;
        Ok(())
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::netblocks::dsl::*;

        diesel::update(netblocks.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::netblocks::dsl::*;

        diesel::update(netblocks.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

impl Netblock {
    // TODO: ips and subnets?
}

pub struct PrintableNetblock {
    value: ipnetwork::IpNetwork,
}

impl fmt::Display for PrintableNetblock {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{}", self.value)
    }
}

impl Printable<PrintableNetblock> for Netblock {
    fn printable(&self, _db: &Database) -> Result<PrintableNetblock> {
        Ok(PrintableNetblock {
            value: self.value.parse()?,
        })
    }
}

pub struct DetailedNetblock {
    id: i32,
    value: ipnetwork::IpNetwork,
    unscoped: bool,
    asn: Option<i32>,
    as_org: Option<String>,
    description: Option<String>,
}

impl DisplayableDetailed for DetailedNetblock {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        w.id(self.id)?;
        w.display::<Green, _>(&self.value)?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.asn)?;
        w.opt_debug::<Yellow, _>(&self.as_org)?;
        w.opt_debug::<Yellow, _>(&self.description)?;
        w.end_group()?;

        Ok(())
    }

    #[inline]
    fn children(&self, _w: &mut fmt::DetailFormatter) -> fmt::Result {
        // TODO: ips, subnets
        Ok(())
    }
}

display_detailed!(DetailedNetblock);

impl Detailed for Netblock {
    type T = DetailedNetblock;

    fn detailed(&self, _db: &Database) -> Result<Self::T> {
        // TODO: ips, subnets
        Ok(DetailedNetblock {
            id: self.id,
            value: self.value.parse()?,
            unscoped: self.unscoped,
            asn: self.asn,
            as_org: self.as_org.clone(),
            description: self.description.clone(),
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="netblocks"]
pub struct NewNetblock {
    pub family: String,
    pub value: String,
    pub asn: Option<i32>,
    pub as_org: Option<String>,
    pub description: Option<String>,
    pub unscoped: bool,
}

impl InsertableStruct<Netblock> for NewNetblock {
    fn value(&self) -> &str {
        &self.value
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.unscoped = !scoped;
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(netblocks::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl Upsertable<Netblock> for NewNetblock {
    type Update = NetblockUpdate;

    fn upsert(self, existing: &Netblock) -> Self::Update {
        Self::Update {
            id: existing.id,
            asn: Self::upsert_opt(self.asn, &existing.asn),
            as_org: Self::upsert_opt(self.as_org, &existing.as_org),
            description: Self::upsert_opt(self.description, &existing.description),
        }
    }
}

impl Printable<PrintableNetblock> for NewNetblock {
    fn printable(&self, _db: &Database) -> Result<PrintableNetblock> {
        Ok(PrintableNetblock {
            value: self.value.parse()?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertNetblock {
    pub value: String,
    pub asn: Option<i32>,
    pub as_org: Option<String>,
    pub description: Option<String>,
}

impl InsertToNew for InsertNetblock {
    type Target = NewNetblock;

    fn try_into_new(self) -> Result<NewNetblock> {
        let ipnet = self.value.parse::<ipnetwork::IpNetwork>()
            .context("Failed to parse ip network")?;

        let family = match ipnet {
            ipnetwork::IpNetwork::V4(_) => "4",
            ipnetwork::IpNetwork::V6(_) => "6",
        };

        Ok(NewNetblock {
            family: family.to_string(),
            value: ipnet.to_string(),
            asn: self.asn,
            as_org: self.as_org,
            description: self.description,
            unscoped: false,
        })
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="netblocks"]
pub struct NetblockUpdate {
    pub id: i32,
    pub asn: Option<i32>,
    pub as_org: Option<String>,
    pub description: Option<String>,
}

impl Upsert for NetblockUpdate {
    fn is_dirty(&self) -> bool {
        self.asn.is_some() ||
        self.as_org.is_some() ||
        self.description.is_some()
    }

    fn generic(self) -> Update {
        Update::Netblock(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_netblock(self)
    }
}

impl Updateable<Netblock> for NetblockUpdate {
    fn changeset(&mut self, existing: &Netblock) {
        Self::clear_if_equal(&mut self.asn, &existing.asn);
        Self::clear_if_equal(&mut self.as_org, &existing.as_org);
        Self::clear_if_equal(&mut self.description, &existing.description);
    }

    fn fmt(&self, updates: &mut Vec<String>, colors: bool) {
        Self::push_value(updates, "asn", &self.asn, colors);
        Self::push_value(updates, "as_org", &self.as_org, colors);
        Self::push_value(updates, "description", &self.description, colors);
    }
}
