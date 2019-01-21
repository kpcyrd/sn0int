use crate::errors::*;

use serde;
use crate::db::Family;
use crate::engine::ctx::State;
use crate::engine::structs;
use crate::hlua::{self, AnyLuaValue};
use std::str::FromStr;
use std::sync::Arc;
use crate::models::*;
use crate::json::LuaJsonValue;


pub fn try_into_new<T: LuaInsertToNewOwned>(x: LuaJsonValue) -> Result<T::Target>
    where for<'de> T: serde::Deserialize<'de>
{
    structs::from_lua::<T>(x)?
        .try_into_new()
}

fn into_insert(family: Family, object: LuaJsonValue) -> Result<Insert> {
    let obj = match family {
        Family::Domain => {
            Insert::Domain(try_into_new::<InsertDomain>(object)?)
        },
        Family::Subdomain => {
            Insert::Subdomain(try_into_new::<InsertSubdomain>(object)?)
        },
        Family::IpAddr => {
            Insert::IpAddr(try_into_new::<InsertIpAddr>(object)?)
        },
        Family::SubdomainIpAddr => {
            Insert::SubdomainIpAddr(try_into_new::<InsertSubdomainIpAddr>(object)?)
        },
        Family::Url => {
            Insert::Url(try_into_new::<InsertUrl>(object)?)
        },
        Family::Email => {
            Insert::Email(try_into_new::<InsertEmail>(object)?)
        },
        Family::PhoneNumber => {
            Insert::PhoneNumber(try_into_new::<InsertPhoneNumber>(object)?)
        },
        Family::Device => {
            Insert::Device(try_into_new::<InsertDevice>(object)?)
        },
        Family::Network => {
            Insert::Network(try_into_new::<InsertNetwork>(object)?)
        },
        Family::NetworkDevice => {
            Insert::NetworkDevice(try_into_new::<InsertNetworkDevice>(object)?)
        },
    };
    Ok(obj)
}

pub fn db_add(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("db_add", hlua::function2(move |family: String, object: AnyLuaValue| -> Result<Option<i32>> {
        let family = Family::from_str(&family)
            .map_err(|e| state.set_error(e))?;
        let object = LuaJsonValue::from(object);

        let object = into_insert(family, object)
            .map_err(|e| state.set_error(e))?;

        state.db_insert(object)
            .map_err(|e| state.set_error(e))
    }))
}

pub fn db_add_ttl(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("db_add_ttl", hlua::function3(move |family: String, object: AnyLuaValue, ttl: i32| -> Result<Option<i32>> {
        let family = Family::from_str(&family)
            .map_err(|e| state.set_error(e))?;
        let object = LuaJsonValue::from(object);

        let object = into_insert(family, object)
            .map_err(|e| state.set_error(e))?;

        state.db_insert_ttl(object, ttl)
            .map_err(|e| state.set_error(e))
    }))
}

pub fn db_select(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("db_select", hlua::function2(move |family: String, value: String| -> Result<Option<i32>> {
        let family = Family::from_str(&family)
            .map_err(|e| state.set_error(e))?;

        state.db_select(family, value)
            .map_err(|e| state.set_error(e))
    }))
}

fn gen_changeset<T: Model, U: Updateable<T>>(object: LuaJsonValue, mut update: LuaJsonValue) -> Result<(i32, String, U)>
    where
        for<'de> T: serde::Deserialize<'de>,
        for<'de> U: serde::Deserialize<'de>,
{
    let existing = structs::from_lua::<T>(object)?;

    // copy the id over to the update struct so we can identify the row
    if let LuaJsonValue::Object(ref mut update) = update {
        update.insert("id".into(), LuaJsonValue::Number(existing.id().into()));
    }

    let mut update = structs::from_lua::<U>(update)?;

    let value = existing.to_string();
    update.changeset(&existing);

    Ok((existing.id(), value, update))
}

pub fn db_update(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("db_update", hlua::function3(move |family: String, object: AnyLuaValue, update: AnyLuaValue| -> Result<Option<i32>> {
        let family = Family::from_str(&family)
            .map_err(|e| state.set_error(e))?;
        let object = LuaJsonValue::from(object);
        let update = LuaJsonValue::from(update);

        let update = match family {
            Family::Domain => bail!("Domain doesn't have mutable fields"),
            Family::Subdomain => gen_changeset::<Subdomain, SubdomainUpdate>(object, update)
                .map(|(id, v, u)| (id, v, Update::Subdomain(u))),
            Family::IpAddr => gen_changeset::<IpAddr, IpAddrUpdate>(object, update)
                .map(|(id, v, u)| (id, v, Update::IpAddr(u))),
            Family::SubdomainIpAddr => bail!("Unsupported operation"),
            Family::Url => gen_changeset::<Url, UrlUpdate>(object, update)
                .map(|(id, v, u)| (id, v, Update::Url(u))),
            Family::Email => gen_changeset::<Email, EmailUpdate>(object, update)
                .map(|(id, v, u)| (id, v, Update::Email(u))),
            Family::PhoneNumber => gen_changeset::<PhoneNumber, PhoneNumberUpdate>(object, update)
                .map(|(id, v, u)| (id, v, Update::PhoneNumber(u))),
            Family::Device => gen_changeset::<Device, DeviceUpdate>(object, update)
                .map(|(id, v, u)| (id, v, Update::Device(u))),
            Family::Network => gen_changeset::<Network, NetworkUpdate>(object, update)
                .map(|(id, v, u)| (id, v, Update::Network(u))),
            Family::NetworkDevice => gen_changeset::<NetworkDevice, NetworkDeviceUpdate>(object, update)
                .map(|(id, v, u)| (id, v, Update::NetworkDevice(u))),
        };

        let (id, value, update) = update
            .map_err(|e| state.set_error(e))?;

        if update.is_dirty() {
            state.db_update(value, update)
                .map_err(|e| state.set_error(e))
        } else {
            Ok(Some(id))
        }
    }))
}
