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


#[cfg(test)]
mod tests {
    use engine::ctx::Script;

    #[test]
    #[ignore]
    fn verify_resolve_a() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('example.com', 'A')
            print(x)
            if x['success'][0]['A'] == nil then
                return "Couldn't resolve"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_resolve_aaaa() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('example.com', 'AAAA')
            print(x)
            if x['success'][0]['AAAA'] == nil then
                return "Couldn't resolve"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_resolve_txt() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('example.com', 'TXT')
            print(x)
            if x['success'][0]['TXT'] == nil then
                return "Couldn't resolve"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_resolve_ns() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('example.com', 'NS')
            print(x)
            if x['success'][0]['NS'] == nil then
                return "Couldn't resolve"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_resolve_soa() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('example.com', 'SOA')
            print(x)
            if x['success'][0]['SOA'] == nil then
                return "Couldn't resolve"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_resolve_ptr() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('1.1.1.1.in-addr.arpa', 'PTR')
            print(x)
            if x['success'][0]['PTR'] == nil then
                return "Couldn't resolve"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
