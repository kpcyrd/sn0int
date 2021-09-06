use crate::db::{Database, Table};
use crate::errors::*;
use crate::models::*;
use crate::notify::{self, Notification};
use crate::schema::*;
use crate::shell::Shell;
use crate::term::{self, Term};
use chrono::{NaiveDateTime, Duration, Utc};
use diesel::prelude::*;
use sn0int_std::ratelimits::Ratelimiter;


#[derive(Identifiable, Queryable, AsChangeset, PartialEq, Debug)]
#[table_name="ttls"]
pub struct Ttl {
    pub id: i32,
    pub family: String,
    pub key: i32,
    pub value: String,
    pub expire: NaiveDateTime,
}

impl Ttl {
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

    pub fn create(obj: &Insert, key: i32, value: String, ttl: i32, db: &Database) -> Result<()> {
        debug!("Creating ttl on record");
        let expire = Self::ttl_to_datetime(ttl);

        diesel::insert_into(ttls::table)
            .values(NewTtl {
                family: obj.table(),
                key,
                value,
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
        let family = self.family.parse::<Table>()?;
        match family {
            Table::Domains => Domain::delete_id(db, self.key)?,
            Table::Subdomains => Subdomain::delete_id(db, self.key)?,
            Table::Ipaddrs => IpAddr::delete_id(db, self.key)?,
            Table::SubdomainIpaddrs => SubdomainIpAddr::delete_id(db, self.key)?,
            Table::Urls => Url::delete_id(db, self.key)?,
            Table::Emails => Email::delete_id(db, self.key)?,
            Table::Phonenumbers => PhoneNumber::delete_id(db, self.key)?,
            Table::Devices => Device::delete_id(db, self.key)?,
            Table::Networks => Network::delete_id(db, self.key)?,
            Table::NetworkDevices => NetworkDevice::delete_id(db, self.key)?,
            Table::Accounts => Account::delete_id(db, self.key)?,
            Table::Breaches => Breach::delete_id(db, self.key)?,
            Table::BreachEmails => BreachEmail::delete_id(db, self.key)?,
            Table::Images => Image::delete_id(db, self.key)?,
            Table::Ports => Port::delete_id(db, self.key)?,
            Table::Netblocks => Netblock::delete_id(db, self.key)?,
            Table::Cryptoaddrs => CryptoAddr::delete_id(db, self.key)?,
        };

        diesel::delete(self)
                .execute(db.db())?;

        Ok(())
    }
}

#[derive(Insertable)]
#[table_name="ttls"]
pub struct NewTtl<'a> {
    pub family: &'a str,
    pub key: i32,
    pub value: String,
    pub expire: NaiveDateTime,
}

pub fn reap_expired(rl: &mut Shell) -> Result<()> {
    debug!("Reaping expired entities");

    let mut ratelimit = Ratelimiter::new();
    for expired in Ttl::expired(rl.db())? {
        debug!("Expired: {:?}", expired);
        expired.delete(rl.db())?;

        let subject = format!("Deleted {} {:?}", &expired.family, &expired.value);
        let topic = &format!("db:{}:{}:delete", &expired.family, &expired.value);
        if let Err(err) = notify::trigger_notify_event(rl, &mut Term, &mut ratelimit, topic, &Notification {
            subject,
            body: None,
        }) {
            term::error(&format!("Failed to send notifications: {}", err));
        }
    }

    debug!("Finished reaping expired entities");
    Ok(())
}
