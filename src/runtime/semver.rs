use crate::engine::ctx::State;
use crate::errors::*;
use crate::hlua;
use semver::{Version, VersionReq};
use std::sync::Arc;


pub fn sn0int_version(lua: &mut hlua::Lua, _: Arc<dyn State>) {
    lua.set("sn0int_version", hlua::function0(move || -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }))
}

pub fn semver_match(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("semver_match", hlua::function2(move |range: String, version: String| -> Result<bool> {
        let r = VersionReq::parse(&range)
            .map_err(|err| state.set_error(Error::from(err)))?;
        let v = Version::parse(&version)
            .map_err(|err| state.set_error(Error::from(err)))?;
        Ok(r.matches(&v))
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_semver_larger() {
        let script = Script::load_unchecked(r#"
        function run()
            v = sn0int_version()
            if not semver_match(">= 0.5.0", v) then
                return 'fail'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_semver_smaller() {
        let script = Script::load_unchecked(r#"
        function run()
            v = sn0int_version()
            if not semver_match("< 999.1.2", v) then
                return 'fail'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_semver_in_range() {
        let script = Script::load_unchecked(r#"
        function run()
            v = sn0int_version()
            if not semver_match(">= 0.7.2, <= 99.9.9", v) then
                return 'fail'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_semver_outer_range() {
        let script = Script::load_unchecked(r#"
        function run()
            v = sn0int_version()
            if semver_match(">= 0.4.2, <= 0.8.9", v) then
                return 'fail'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
