use crate::errors::*;
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use crate::models::*;
use chrono::NaiveDateTime;

#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[belongs_to(Network)]
#[belongs_to(Device)]
#[table_name="network_devices"]
pub struct NetworkDevice {
    pub id: i32,
    pub network_id: i32,
    pub device_id: i32,
    pub ipaddr: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
}

impl Model for NetworkDevice {
    type ID = (i32, i32);

    fn to_string(&self) -> String {
        unimplemented!("NetworkDevice can not be printed")
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::network_devices::dsl::*;

        let results = network_devices.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::network_devices::dsl::*;

        let query = network_devices.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::network_devices::dsl::*;

        diesel::delete(network_devices.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::network_devices::dsl::*;

        diesel::delete(network_devices.filter(id.eq(my_id)))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn id(&self) -> i32 {
        self.id
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use crate::schema::network_devices::dsl::*;

        let network_device = network_devices.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(network_device)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::network_devices::dsl::*;

        let (my_network_id, my_device_id) = query;
        let network_device = network_devices.filter(network_id.eq(my_network_id))
                                                   .filter(device_id.eq(my_device_id))
                                                   .first::<Self>(db.db())?;

        Ok(network_device)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::network_devices::dsl::*;

        let (my_network_id, my_device_id) = query;
        let network_device = network_devices.filter(network_id.eq(my_network_id))
                                                   .filter(device_id.eq(my_device_id))
                                                   .first::<Self>(db.db())
                                                   .optional()?;

        Ok(network_device)
    }
}

impl NetworkDevice {
    pub fn network(&self, db: &Database) -> Result<Network> {
        Network::by_id(db, self.network_id)
    }
}

pub struct PrintableNetworkDevice {
    network: String,
    device: String,
}

impl fmt::Display for PrintableNetworkDevice {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?} -> {:?}", self.network, self.device)
    }
}

impl Printable<PrintableNetworkDevice> for NetworkDevice {
    fn printable(&self, db: &Database) -> Result<PrintableNetworkDevice> {
        let network = Network::by_id(db, self.network_id)?;
        let device = Device::by_id(db, self.device_id)?;
        Ok(PrintableNetworkDevice {
            network: network.value,
            device: device.value,
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="network_devices"]
pub struct NewNetworkDevice {
    pub network_id: i32,
    pub device_id: i32,
    pub ipaddr: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
}

impl Upsertable<NetworkDevice> for NewNetworkDevice {
    type Update = NetworkDeviceUpdate;

    fn upsert(self, existing: &NetworkDevice) -> Self::Update {
        Self::Update {
            id: existing.id,
            ipaddr: Self::upsert_opt(self.ipaddr, &existing.ipaddr),
            last_seen: Self::upsert_opt(self.last_seen, &existing.last_seen),
        }
    }
}

impl Printable<PrintableNetworkDevice> for NewNetworkDevice {
    fn printable(&self, db: &Database) -> Result<PrintableNetworkDevice> {
        let network = Network::by_id(db, self.network_id)?;
        let device = Device::by_id(db, self.device_id)?;
        Ok(PrintableNetworkDevice {
            network: network.value,
            device: device.value,
        })
    }
}

pub type InsertNetworkDevice = NewNetworkDevice;

impl InsertToNew for InsertNetworkDevice {
    type Target = NewNetworkDevice;

    #[inline]
    fn try_into_new(self) -> Result<NewNetworkDevice> {
        Ok(self)
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="network_devices"]
pub struct NetworkDeviceUpdate {
    pub id: i32,
    pub ipaddr: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
}

impl Upsert for NetworkDeviceUpdate {
    fn is_dirty(&self) -> bool {
        self.ipaddr.is_some() ||
        self.last_seen.is_some()
    }

    fn generic(self) -> Update {
        Update::NetworkDevice(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_network_device(self)
    }
}

impl Updateable<NetworkDevice> for NetworkDeviceUpdate {
    fn changeset(&mut self, existing: &NetworkDevice) {
        Self::clear_if_equal(&mut self.ipaddr, &existing.ipaddr);
        Self::clear_if_equal(&mut self.last_seen, &existing.last_seen);
    }

    fn fmt(&self, updates: &mut Vec<String>, colors: bool) {
        Self::push_value(updates, "ipaddr", &self.ipaddr, colors);
        Self::push_value(updates, "last_seen", &self.last_seen, colors);
    }
}
