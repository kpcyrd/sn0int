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

pub fn db_update(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("db_update", hlua::function3(move |family: String, object: AnyLuaValue, update: AnyLuaValue| -> Result<i32> {
        let family = Family::from_str(&family)
            .map_err(|e| state.set_error(e))?;
        let object = LuaJsonValue::from(object);
        let mut update = LuaJsonValue::from(update);

        let (id, object) = match family {
            Family::Domain => bail!("Domain doesn't have mutable fields"),
            Family::Subdomain => structs::from_lua::<Domain>(object)
                    .map(|x| (x.id, x.to_string()))
                    .map_err(|e| state.set_error(e)),
            Family::IpAddr => structs::from_lua::<IpAddr>(object)
                    .map(|x| (x.id, x.to_string()))
                    .map_err(|e| state.set_error(e)),
            Family::SubdomainIpAddr => bail!("Unsupported operation"),
            Family::Url => structs::from_lua::<Url>(object)
                    .map(|x| (x.id, x.to_string()))
                    .map_err(|e| state.set_error(e)),
            Family::Email => structs::from_lua::<Email>(object)
                    .map(|x| (x.id, x.to_string()))
                    .map_err(|e| state.set_error(e)),
        }?;

        // copy the id over to the update struct so we can identify the row
        if let LuaJsonValue::Object(ref mut update) = update {
            update.insert("id".into(), LuaJsonValue::Number(id.into()));
        }

        let update = match family {
            Family::Domain => bail!("Domain doesn't have mutable fields"),
            Family::Subdomain => Update::Subdomain(structs::from_lua::<SubdomainUpdate>(update)
                .map_err(|e| state.set_error(e))?),
            Family::IpAddr => Update::IpAddr(structs::from_lua::<IpAddrUpdate>(update)
                .map_err(|e| state.set_error(e))?),
            Family::SubdomainIpAddr => bail!("Unsupported operation"),
            Family::Url => Update::Url(structs::from_lua::<UrlUpdate>(update)
                .map_err(|e| state.set_error(e))?),
            Family::Email => Update::Email(structs::from_lua::<EmailUpdate>(update)
                .map_err(|e| state.set_error(e))?),
        };

        state.db_update(object, update)
            .map_err(|e| state.set_error(e))
    }))
}
