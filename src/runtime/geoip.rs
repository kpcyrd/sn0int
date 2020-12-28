use crate::errors::*;

use crate::engine::ctx::State;
use crate::json::LuaJsonValue;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;
use std::net::IpAddr;


pub fn asn_lookup(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("asn_lookup", hlua::function1(move |ip: String| -> Result<AnyLuaValue> {
        let asn = state.asn()
            .map_err(|err| state.set_error(err))?;

        let ip = ip.parse::<IpAddr>()
            .context("Failed to parse IP")
            .map_err(|err| state.set_error(err.into()))?;

        let lookup = asn.lookup(ip)
            .map_err(|err| state.set_error(err))?;

        let lookup = serde_json::to_value(lookup)
            .map_err(|e| state.set_error(e.into()))?;

        Ok(LuaJsonValue::from(lookup).into())
    }))
}

pub fn geoip_lookup(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("geoip_lookup", hlua::function1(move |ip: String| -> Result<AnyLuaValue> {
        let geoip = state.geoip()
            .map_err(|err| state.set_error(err))?;

        let ip = ip.parse::<IpAddr>()
            .context("Failed to parse IP")
            .map_err(|err| state.set_error(err.into()))?;

        let lookup = geoip.lookup(ip)
            .map_err(|err| state.set_error(err))?;

        let lookup = serde_json::to_value(lookup)
            .map_err(|e| state.set_error(e.into()))?;

        Ok(LuaJsonValue::from(lookup).into())
    }))
}


#[cfg(test)]
mod tests {
    // You need geoip setup on your system to run this
    /*
    use crate::engine::ctx::Script;

    #[test]
    fn verify_asn_lookup() {
        let script = Script::load_unchecked(r#"
        function run()
            x = asn_lookup('1.1.1.1')
            print(x)
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_geoip_lookup() {
        let script = Script::load_unchecked(r#"
        function run()
            x = geoip_lookup('1.1.1.1')
            print(x)
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
    */
}
