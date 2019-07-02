use crate::errors::*;
use crate::db::Database;
use crate::schema::*;
use crate::models::*;
use chrono::{NaiveDateTime, Duration, Utc};
use diesel;
use diesel::prelude::*;


#[derive(Identifiable, Queryable, AsChangeset, PartialEq, Debug)]
#[table_name="ttls"]
pub struct Ttl {
    pub id: i32,
    pub family: String,
    pub key: i32,
    pub expire: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name="ttls"]
pub struct NewTtl<'a> {
    pub family: &'a str,
    pub key: i32,
    pub expire: NaiveDateTime,
}

impl Ttl {
    pub fn new(obj: &Insert, key: i32, expire: NaiveDateTime) -> NewTtl {
        NewTtl {
            family: obj.table(),
            key,
            expire,
        }
    }

    pub fn find(obj: &Insert, my_key: i32, db: &Database) -> Result<Option<Ttl>> {
        use crate::schema::ttls::dsl::*;

        ttls
            .filter(family.eq(obj.table()))
            .filter(key.eq(my_key))
            .first::<Self>(db.db())
            .optional()
            .map_err(Error::from)
    }

    pub fn expired(db: &Database) -> Result<Vec<Ttl>> {
        use crate::schema::ttls::dsl::*;

        ttls
            .filter(expire.lt(Self::ttl_to_datetime(0)))
            .load::<Self>(db.db())
            .map_err(Error::from)
    }

    fn ttl_to_datetime(ttl: i32) -> NaiveDateTime {
        // TODO: maybe create Duration from string
        let expire_at = Utc::now() + Duration::seconds(ttl as i64);
        expire_at.naive_utc()
    }

    pub fn create(obj: &Insert, key: i32, ttl: i32, db: &Database) -> Result<()> {
        debug!("Creating ttl on record");
        let expire = Self::ttl_to_datetime(ttl);

        diesel::insert_into(ttls::table)
            .values(NewTtl {
                family: obj.table(),
                key,
                expire,
            })
            .execute(db.db())?;

        Ok(())
    }

    pub fn bump(obj: &Insert, my_key: i32, ttl: i32, db: &Database) -> Result<()> {
        use crate::schema::ttls::dsl::*;

        debug!("Updating ttl on record");

        if let Some(mut old) = Self::find(obj, my_key, db)? {
            let new_expire = Self::ttl_to_datetime(ttl);

            if old.expire < new_expire {
                debug!("Bumping old expire date");

                old.expire = new_expire;

                diesel::update(ttls.filter(id.eq(old.id)))
                    .set(old)
                    .execute(db.db())?;
            }
        } else {
            debug!("Existing record doesn't expire, not setting a ttl");
        }

        Ok(())
    }

    pub fn delete(&self, db: &Database) -> Result<()> {
        match self.family.as_str() {
            "domains" => Domain::delete_id(db, self.key)?,
            "subdomains" => Subdomain::delete_id(db, self.key)?,
            "ipaddrs" => IpAddr::delete_id(db, self.key)?,
            "subdomain_ipaddrs" => SubdomainIpAddr::delete_id(db, self.key)?,
            "urls" => Url::delete_id(db, self.key)?,
            "emails" => Email::delete_id(db, self.key)?,
            "phonenumbers" => PhoneNumber::delete_id(db, self.key)?,
            "devices" => Device::delete_id(db, self.key)?,
            "networks" => Network::delete_id(db, self.key)?,
            "network_devices" => NetworkDevice::delete_id(db, self.key)?,
            "accounts" => Account::delete_id(db, self.key)?,
            "breaches" => Breach::delete_id(db, self.key)?,
            "breach_emails" => BreachEmail::delete_id(db, self.key)?,
            "images" => Image::delete_id(db, self.key)?,
            "ports" => Port::delete_id(db, self.key)?,
            "netblocks" => Netblock::delete_id(db, self.key)?,
            _ => bail!("Unknown table"),
        };

        diesel::delete(self)
                .execute(db.db())?;

        Ok(())
    }
}

pub fn reap_expired(db: &Database) -> Result<()> {
    debug!("Reaping expired entities");

    for expired in Ttl::expired(db)? {
        debug!("Expired: {:?}", expired);
        expired.delete(db)?;
    }

    debug!("Finished reaping expired entities");
    Ok(())
}
