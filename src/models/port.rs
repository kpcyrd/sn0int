use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::fmt::Write;
use crate::fmt::colors::*;
use crate::models::*;
use diesel::prelude::*;
use std::net::{self, SocketAddr};

#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(IpAddr)]
#[table_name="ports"]
pub struct Port {
    pub id: i32,
    pub ip_addr_id: i32,
    pub value: String,
    pub ip_addr: String,
    pub port: i32,
    pub protocol: String,
    pub status: Option<String>,
    pub unscoped: bool,

    pub banner: Option<String>,
    pub service: Option<String>,
    pub version: Option<String>,
}

impl Model for Port {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::ports::dsl::*;

        let results = ports.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::ports::dsl::*;

        let query = ports.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::ports::dsl::*;

        diesel::delete(ports.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::ports::dsl::*;

        diesel::delete(ports.filter(id.eq(my_id)))
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
        use crate::schema::ports::dsl::*;

        let url = ports.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(url)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::ports::dsl::*;

        let url = ports.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(url)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::ports::dsl::*;

        let url = ports.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(url)
    }
}

impl Scopable for Port {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn set_scoped(&self, db: &Database, my_value: bool) -> Result<()> {
        use crate::schema::ports::dsl::*;
        diesel::update(ports.filter(id.eq(self.id)))
            .set(unscoped.eq(!my_value))
            .execute(db.db())?;
        Ok(())
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::ports::dsl::*;

        diesel::update(ports.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::ports::dsl::*;

        diesel::update(ports.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

pub struct PrintablePort {
    value: String,
}

impl fmt::Display for PrintablePort {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintablePort> for Port {
    fn printable(&self, _db: &Database) -> Result<PrintablePort> {
        Ok(PrintablePort {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedPort {
    id: i32,
    value: String,
    status: Option<String>,
    unscoped: bool,

    banner: Option<String>,
    service: Option<String>,
    version: Option<String>,
}

impl DisplayableDetailed for DetailedPort {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        w.id(self.id)?;
        w.display::<Green, _>(&self.value)?;
        write!(w, ", ")?;
        w.debug::<Yellow, _>(&self.status)?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.banner)?;
        w.opt_debug::<Yellow, _>(&self.service)?;
        w.opt_debug::<Yellow, _>(&self.version)?;
        w.end_group()?;

        Ok(())
    }

    #[inline]
    fn children(&self, _w: &mut fmt::DetailFormatter) -> fmt::Result {
        Ok(())
    }
}

display_detailed!(DetailedPort);

impl Detailed for Port {
    type T = DetailedPort;

    fn detailed(&self, _db: &Database) -> Result<Self::T> {
        Ok(DetailedPort {
            id: self.id,
            value: self.value.clone(),
            status: self.status.clone(),
            unscoped: self.unscoped,

            banner: self.banner.clone(),
            service: self.service.clone(),
            version: self.version.clone(),
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="ports"]
pub struct NewPort {
    pub ip_addr_id: i32,
    pub value: String,
    pub ip_addr: String,
    pub port: i32,
    pub protocol: String,
    pub status: Option<String>,

    pub banner: Option<String>,
    pub service: Option<String>,
    pub version: Option<String>,

    pub unscoped: bool,
}

impl InsertableStruct<Port> for NewPort {
    fn value(&self) -> &str {
        &self.value
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.unscoped = !scoped;
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(ports::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl Upsertable<Port> for NewPort {
    type Update = PortUpdate;

    fn upsert(self, existing: &Port) -> Self::Update {
        Self::Update {
            id: existing.id,
            status: Self::upsert_opt(self.status, &existing.status),
            banner: Self::upsert_opt(self.banner, &existing.banner),
            service: Self::upsert_opt(self.service, &existing.service),
            version: Self::upsert_opt(self.version, &existing.version),
        }
    }
}

impl Printable<PrintablePort> for NewPort {
    fn printable(&self, _db: &Database) -> Result<PrintablePort> {
        Ok(PrintablePort {
            value: self.value.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertPort {
    pub ip_addr_id: i32,
    pub ip_addr: net::IpAddr,
    pub port: i32,
    pub protocol: String,
    pub status: Option<String>,

    pub banner: Option<String>,
    pub service: Option<String>,
    pub version: Option<String>,
}

impl InsertToNew for InsertPort {
    type Target = NewPort;

    fn try_into_new(self) -> Result<NewPort> {
        let addr = SocketAddr::new(self.ip_addr, self.port as u16);
        let value = format!("{}/{}", self.protocol, addr);

        match self.status.as_deref() {
            Some("open") => (),
            Some("closed") => (),
            Some(s) => bail!("unsupported port status: {:?}", s),
            None => (),
        }

        Ok(NewPort {
            ip_addr_id: self.ip_addr_id,
            value,
            ip_addr: self.ip_addr.to_string(),
            port: self.port,
            protocol: self.protocol,
            status: self.status,

            banner: self.banner,
            service: self.service,
            version: self.version,

            unscoped: false,
        })
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="ports"]
pub struct PortUpdate {
    pub id: i32,
    pub status: Option<String>,
    pub banner: Option<String>,
    pub service: Option<String>,
    pub version: Option<String>,
}

impl Upsert for PortUpdate {
    fn is_dirty(&self) -> bool {
        self.status.is_some() ||
        self.banner.is_some() ||
        self.service.is_some() ||
        self.version.is_some()
    }

    fn generic(self) -> Update {
        Update::Port(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_port(self)
    }
}

impl Updateable<Port> for PortUpdate {
    fn changeset(&mut self, existing: &Port) {
        Self::clear_if_equal(&mut self.status, &existing.status);
        Self::clear_if_equal(&mut self.banner, &existing.banner);
        Self::clear_if_equal(&mut self.service, &existing.service);
        Self::clear_if_equal(&mut self.version, &existing.version);
    }

    fn fmt(&self, updates: &mut Vec<String>, colors: bool) {
        Self::push_value(updates, "status", &self.status, colors);
        Self::push_value(updates, "banner", &self.banner, colors);
        Self::push_value(updates, "service", &self.service, colors);
        Self::push_value(updates, "version", &self.version, colors);
    }
}
