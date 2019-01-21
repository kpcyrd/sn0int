use crate::errors::*;
use crate::fmt::colors::*;
use diesel;
use diesel::prelude::*;
use crate::models::*;
use chrono::NaiveDateTime;


#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="devices"]
pub struct Device {
    pub id: i32,
    pub value: String,
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub vendor: Option<String>,
    pub unscoped: bool,
    pub last_seen: Option<NaiveDateTime>,
}

impl Model for Device {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::devices::dsl::*;

        let results = devices.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::devices::dsl::*;

        let query = devices.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::devices::dsl::*;

        diesel::delete(devices.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::devices::dsl::*;

        diesel::delete(devices.filter(id.eq(my_id)))
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
        use crate::schema::devices::dsl::*;

        let domain = devices.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::devices::dsl::*;

        let domain = devices.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::devices::dsl::*;

        let domain = devices.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(domain)
    }
}

impl Scopable for Device {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::devices::dsl::*;

        diesel::update(devices.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::devices::dsl::*;

        diesel::update(devices.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

impl Device {
    fn network_device(&self, db: &Database) -> Result<Option<NetworkDevice>> {
        NetworkDevice::belonging_to(self)
            .order(network_devices::last_seen.desc())
            .first::<NetworkDevice>(db.db())
            .optional()
            .map_err(Error::from)
    }
}

pub struct PrintableDevice {
    value: String,
}

impl fmt::Display for PrintableDevice {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintableDevice> for Device {
    fn printable(&self, _db: &Database) -> Result<PrintableDevice> {
        Ok(PrintableDevice {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedDevice {
    id: i32,
    value: String,
    name: Option<String>,
    hostname: Option<String>,
    vendor: Option<String>,
    ipaddr: Option<String>,
    network: Option<String>,
    unscoped: bool,
    last_seen: Option<NaiveDateTime>,
}

impl DisplayableDetailed for DetailedDevice {
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
        w.opt_debug::<Yellow, _>(&self.hostname)?;
        w.opt_debug::<Yellow, _>(&self.vendor)?;
        w.opt_debug::<Yellow, _>(&self.last_seen)?;
        w.end_group()?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.network)?;
        w.opt_debug::<Yellow, _>(&self.ipaddr)?;
        w.end_group()?;

        Ok(())
    }

    #[inline]
    fn children(&self, _w: &mut fmt::DetailFormatter) -> fmt::Result {
        Ok(())
    }
}

display_detailed!(DetailedDevice);

impl Detailed for Device {
    type T = DetailedDevice;

    fn detailed(&self, db: &Database) -> Result<Self::T> {
        let network_device = self.network_device(db)?;

        let (ipaddr, network) = match network_device {
            Some(network_device) => {
                let network = network_device.network(db)?;
                (network_device.ipaddr, Some(network.value))
            },
            _ => (None, None),
        };

        Ok(DetailedDevice {
            id: self.id,
            value: self.value.to_string(),
            name: self.name.clone(),
            hostname: self.hostname.clone(),
            vendor: self.vendor.clone(),
            ipaddr,
            network,
            unscoped: self.unscoped,
            last_seen: self.last_seen.clone(),
        })
    }
}

#[derive(Insertable)]
#[table_name="devices"]
pub struct NewDevice<'a> {
    pub value: &'a str,
    pub name: Option<&'a String>,
    pub hostname: Option<&'a String>,
    pub vendor: Option<&'a String>,
    pub last_seen: Option<NaiveDateTime>,
}

impl<'a> InsertableStruct<Device> for NewDevice<'a> {
    fn value(&self) -> &str {
        self.value
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(devices::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl<'a> Upsertable<Device> for NewDevice<'a> {
    type Update = DeviceUpdate;

    fn upsert(self, existing: &Device) -> Self::Update {
        Self::Update {
            id: existing.id,
            name: Self::upsert_str(self.name, &existing.name),
            hostname: Self::upsert_str(self.hostname, &existing.hostname),
            vendor: Self::upsert_str(self.vendor, &existing.vendor),
            last_seen: Self::upsert_opt(self.last_seen, &existing.last_seen),
        }
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="devices"]
pub struct NewDeviceOwned {
    pub value: String,
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub vendor: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
}

impl Printable<PrintableDevice> for NewDeviceOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableDevice> {
        Ok(PrintableDevice {
            value: self.value.to_string(),
        })
    }
}

pub type InsertDevice = NewDeviceOwned;

impl LuaInsertToNewOwned for InsertDevice {
    type Target = NewDeviceOwned;

    fn try_into_new(self) -> Result<NewDeviceOwned> {
        Ok(self)
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="devices"]
pub struct DeviceUpdate {
    pub id: i32,
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub vendor: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
}

impl Upsert for DeviceUpdate {
    fn is_dirty(&self) -> bool {
        self.name.is_some() ||
        self.hostname.is_some() ||
        self.vendor.is_some() ||
        self.last_seen.is_some()
    }

    fn generic(self) -> Update {
        Update::Device(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_device(self)
    }
}

impl Updateable<Device> for DeviceUpdate {
    fn changeset(&mut self, existing: &Device) {
        Self::clear_if_equal(&mut self.name, &existing.name);
        Self::clear_if_equal(&mut self.hostname, &existing.hostname);
        Self::clear_if_equal(&mut self.vendor, &existing.vendor);
        Self::clear_if_equal(&mut self.last_seen, &existing.last_seen);
    }

    fn fmt(&self, updates: &mut Vec<String>) {
        Self::push_value(updates, "name", &self.name);
        Self::push_value(updates, "hostname", &self.hostname);
        Self::push_value(updates, "vendor", &self.vendor);
        Self::push_value(updates, "last_seen", &self.last_seen);
    }
}
