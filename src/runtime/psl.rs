use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua;
use std::sync::Arc;

pub fn psl_domain_from_dns_name(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("psl_domain_from_dns_name", hlua::function1(move |dns_name: String| -> Result<String> {
        let dns_name = state.psl().parse_dns_name(&dns_name)
            .map_err(|err| state.set_error(err))?;

        let domain = dns_name.domain()
            .ok_or_else(|| format_err!("Failed to get domain from {:?}", dns_name))
            .map_err(|err| state.set_error(err))?
            .to_string();

        Ok(domain)
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
                return 'unexpected domain'
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
                return 'unexpected domain'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_psl_lookup_invalid() {
        let script = Script::load_unchecked(r#"
        function run()
            domain = psl_domain_from_dns_name("asdf")
            if last_err() then
                clear_err()
            else
                return "invalid value didn't cause error"
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
