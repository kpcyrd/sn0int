use errors::*;

use engine::ctx::State;
use hlua::{self, AnyLuaValue};
use std::str::FromStr;
use std::sync::Arc;
use models::*;
// use hlua::AnyHashableLuaValue;
// use std::collections::HashMap;
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
                Insert::Domain(NewDomainOwned::from_lua(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::Subdomain => {
                Insert::Subdomain(NewSubdomainOwned::from_lua(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::IpAddr => {
                Insert::IpAddr(NewIpAddrOwned::from_lua(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::SubdomainIpAddr => {
                Insert::SubdomainIpAddr(NewSubdomainIpAddr::from_lua(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::Url => {
                Insert::Url(NewUrlOwned::from_lua(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::Email => {
                Insert::Email(NewEmailOwned::from_lua(object)
                    .map_err(|e| state.set_error(e))?)
            },
        };

        state.db_insert(object)
            .map_err(|e| state.set_error(e))
    }))
}

pub fn db_update(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("db_update", hlua::function2(move |family: String, object: AnyLuaValue| -> Result<i32> {
        let family = Family::from_str(&family)
            .map_err(|e| state.set_error(e))?;
        let object = LuaJsonValue::from(object);

        let object = match family {
            Family::Domain => bail!("Domain doesn't have mutable fields"),
            Family::Subdomain => {
                Update::Subdomain(SubdomainUpdate::from_lua(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::IpAddr => bail!("IpAddr doesn't have mutable fields"),
            Family::SubdomainIpAddr => bail!("Unsupported operation"),
            Family::Url => {
                Update::Url(UrlUpdate::from_lua(object)
                    .map_err(|e| state.set_error(e))?)
            },
            Family::Email => {
                Update::Email(EmailUpdate::from_lua(object)
                    .map_err(|e| state.set_error(e))?)
            },
        };

        state.db_update(object)
            .map_err(|e| state.set_error(e))
    }))
}
