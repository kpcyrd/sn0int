use crate::errors::*;
use serde::Deserialize;

use chrootable_https::dns::{Resolver, DnsResolver, RecordType};
use crate::engine;
use crate::engine::ctx::State;
use crate::json::LuaJsonValue;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;
use std::net::SocketAddr;
use std::time::Duration;


#[derive(Debug, Deserialize)]
pub struct ResolveOptions {
    //record: RecordType,
    record: String,
    // TODO: this could be more than one
    nameserver: Option<SocketAddr>,
    #[serde(default)]
    tcp: bool,
    timeout: Option<u64>,
}

impl ResolveOptions {
    pub fn new(record: &str) -> Result<ResolveOptions> {
        let record = record.parse()?;
        Ok(ResolveOptions {
            record,
            nameserver: None,
            tcp: false,
            timeout: None,
        })
    }

    pub fn record_type(&self) -> Result<RecordType> {
        self.record.parse()
            .map_err(Error::from)
    }

    pub fn from_lua(x: AnyLuaValue) -> Result<ResolveOptions> {
        match x {
            AnyLuaValue::LuaAnyString(s) => {
                let s = String::from_utf8(s.0)?;
                ResolveOptions::new(&s)
            },
            AnyLuaValue::LuaString(s) => ResolveOptions::new(&s),
            x => engine::structs::from_lua(x.into()),
        }
    }
}

pub fn dns(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("dns", hlua::function2(move |name: String, options: AnyLuaValue| -> Result<AnyLuaValue> {
        if state.proxy().is_some() {
            let e = format_err!("dns is disabled if a proxy is active");
            return Err(state.set_error(e));
        }

        let options = ResolveOptions::from_lua(options)
            .map_err(|e| state.set_error(e))?;

        let ns = match options.nameserver {
            Some(ns) => vec![ns],
            None => state.dns_config().ns.clone(),
        };

        let timeout = options.timeout
            .map(Duration::from_millis);

        let resolver = Resolver {
            ns,
            tcp: options.tcp,
            timeout,
        };

        let reply = resolver.resolve(&name, options.record_type()?)
            .wait_for_response()
            .map_err(|e| state.set_error(e))?;

        let reply = serde_json::to_value(reply)
            .map_err(|e| state.set_error(e.into()))?;

        Ok(LuaJsonValue::from(reply).into())
    }))
}


#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    #[ignore]
    fn verify_resolve_a() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('example.com', 'A')
            if last_err() then return end
            print(x)
            if x['answers'][1][2]['A'] == nil then
                return "Couldn't resolve"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_custom_resolver_a() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('example.com', {
                record='A',
                nameserver='1.1.1.1:53',
            })
            if last_err() then return end
            print(x)
            if x['answers'][1][2]['A'] == nil then
                return "Couldn't resolve"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_custom_resolver_axfr() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('zonetransfer.me', {
                record='AXFR',
                nameserver='81.4.108.41:53',
                tcp=true,
                timeout=15000,
            })
            if last_err() then return end
            print(x)

            if x['error'] ~= nil then
                return 'Error: ' .. x['error']
            end

            has_soa = false
            has_ns = false
            has_a = false
            has_mx = false

            i = 1
            answers = x['answers']
            while answers[i] do
                r = answers[i][2]

                if r['SOA'] ~= nil then
                    has_soa = true
                end

                if r['NS'] ~= nil then
                    has_ns = true
                end

                if r['A'] ~= nil then
                    has_a = true
                end

                if r['MX'] ~= nil then
                    has_mx = true
                end

                i = i+1
            end

            if not has_soa then
                return 'Error: missing soa'
            end

            if not has_ns then
                return 'Error: missing ns'
            end

            if not has_a then
                return 'Error: missing a'
            end

            if not has_mx then
                return 'Error: missing mx'
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
            if last_err() then return end
            print(x)
            if x['answers'][1][2]['AAAA'] == nil then
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
            if last_err() then return end
            print(x)
            if x['answers'][1][2]['TXT'] == nil then
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
            if last_err() then return end
            print(x)
            if x['answers'][1][2]['NS'] == nil then
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
            if last_err() then return end
            print(x)
            if x['answers'][1][2]['SOA'] == nil then
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
            if last_err() then return end
            print(x)
            if x['answers'][1][2]['PTR'] == nil then
                return "Couldn't resolve"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_nx_record() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('doesntexist.example.com', 'A')
            if last_err() then return end
            print(x)
            if x['error'] ~= 'NXDomain' then
                return "Expected NXDomain"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_empty_cname() {
        let script = Script::load_unchecked(r#"
        function run()
            x = dns('example.com', 'CNAME')
            if last_err() then return end
            print(x)
            if x['error'] ~= nil then
                return "Expected no error"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
