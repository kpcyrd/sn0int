use errors::*;

use engine::ctx::State;
use hlua::{self, AnyLuaValue};
use std::sync::Arc;
use models::*;
// use hlua::AnyHashableLuaValue;
// use std::collections::HashMap;
use json::LuaJsonValue;


pub fn db_add(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("db_add", hlua::function2(move |family: String, object: AnyLuaValue| -> Result<()> {
        let object = LuaJsonValue::from(object);

        let object = match family.as_str() {
            "subdomain" => {
                Object::Subdomain(NewSubdomainOwned::from_lua(object)?)
            },
            "ipaddr" => {
                Object::IpAddr(NewIpAddrOwned::from_lua(object)?)
            },
            "subdomain-ipaddr" => {
                Object::SubdomainIpAddr(NewSubdomainIpAddr::from_lua(object)?)
            },
            _ => return Err(state.set_error(format_err!("Unknown object family"))),
        };

        state.db_insert(object);
        Ok(())
    }))
}
