use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua;
use std::sync::Arc;

pub fn psl_domain_from_dns_name(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("psl_domain_from_dns_name", hlua::function1(move |dns_name: String| -> Result<String> {
        let psl = state.psl()
            .map_err(|err| state.set_error(err))?;

        let dns_name = psl.parse_dns_name(&dns_name)
            .map_err(|err| state.set_error(err))?;

        Ok(dns_name.root)
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_psl_lookup() {
        let script = Script::load_unchecked(r#"
        function run()
            domain = psl_domain_from_dns_name("foo.example.com")
            if domain ~= "example.com" then 
                return 'unexpected domain: ' .. domain
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_psl_lookup_no_subdomain() {
        let script = Script::load_unchecked(r#"
        function run()
            domain = psl_domain_from_dns_name("example.com")
            if domain ~= "example.com" then 
                return 'unexpected domain: ' .. domain
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_psl_lookup_tld() {
        let script = Script::load_unchecked(r#"
        function run()
            domain = psl_domain_from_dns_name("asdfinvalid")
            if domain ~= "asdfinvalid" then
                return 'unexpected domain: ' .. domain
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_psl_lookup_fastly() {
        let script = Script::load_unchecked(r#"
        function run()
            domain = psl_domain_from_dns_name("a.prod.fastly.net")
            if domain ~= "a.prod.fastly.net" then
                return 'unexpected domain: ' .. domain
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
