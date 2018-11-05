use errors::*;

use engine::ctx::State;
use engine::structs;
use hlua::{self, AnyLuaValue};
use std::str::FromStr;
use std::sync::Arc;
use models::*;
use json::LuaJsonValue;


#[derive(Debug)]
pub enum Family {
    Domain,
    Subdomain,
    IpAddr,
    SubdomainIpAddr,
    Url,
    Email,
}

impl FromStr for Family {
    type Err = Error;

    fn from_str(s: &str) -> Result<Family> {
        Ok(match s {
            "domain" => Family::Domain,
            "subdomain" => Family::Subdomain,
            "ipaddr" => Family::IpAddr,
            "subdomain-ipaddr" => Family::SubdomainIpAddr,
            "url" => Family::Url,
            "email" => Family::Email,
            _ => bail!("Unknown object family"),
        })
    }
}

pub fn db_add(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("db_add", hlua::function2(move |family: String, object: AnyLuaValue| -> Result<i32> {
        let family = Family::from_str(&family)
            .map_err(|e| state.set_error(e))?;
        let object = LuaJsonValue::from(object);

        let object = match family {
            Family::Domain => {
                Insert::Domain(structs::from_lua::<NewDomainOwned>(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::Subdomain => {
                Insert::Subdomain(structs::from_lua::<NewSubdomainOwned>(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::IpAddr => {
                Insert::IpAddr(structs::from_lua::<NewIpAddrOwned>(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::SubdomainIpAddr => {
                Insert::SubdomainIpAddr(structs::from_lua::<NewSubdomainIpAddr>(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::Url => {
                Insert::Url(structs::from_lua::<NewUrlOwned>(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::Email => {
                Insert::Email(structs::from_lua::<NewEmailOwned>(object)
                    .map_err(|e| state.set_error(e))?)
            },
        };

        state.db_insert(object)
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
    lua.set("db_update", hlua::function3(move |family: String, object: AnyLuaValue, update: AnyLuaValue| -> Result<i32> {
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
        };

        let (id, value, update) = update
            .map_err(|e| state.set_error(e))?;

        if update.is_dirty() {
            state.db_update(value, update)
                .map_err(|e| state.set_error(e))
        } else {
            Ok(id)
        }
    }))
}
