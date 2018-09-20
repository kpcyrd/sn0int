use errors::*;

use chrootable_https::{Resolver, DnsResolver};
use engine::ctx::State;
use json::LuaJsonValue;
use hlua::{self, AnyLuaValue};
use serde_json;
use std::sync::Arc;
use trust_dns_proto;


pub fn dns(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("dns", hlua::function2(move |name: String, record: String| -> Result<AnyLuaValue> {
        let resolver = Resolver::from_system()
            .map_err(|e| state.set_error(e))?;

        let record = record.parse()
            .map_err(|e: trust_dns_proto::error::ProtoError| state.set_error(e.into()))?;

        let reply = resolver.resolve_adv(&name, record)
            .map_err(|e| state.set_error(e))?;

        let reply = serde_json::to_value(reply)
            .map_err(|e| state.set_error(e.into()))?;

        Ok(LuaJsonValue::from(reply).into())
    }))
}
