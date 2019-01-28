use crate::engine::ctx::State;
use crate::hlua;
use chrono::{Utc};
use std::sync::Arc;


pub fn datetime(lua: &mut hlua::Lua, _: Arc<State>) {
    lua.set("datetime", hlua::function0(move || -> String {
        let now = Utc::now().naive_utc();
        now.format("%Y-%m-%dT%H:%M:%S")
           .to_string()
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_datetime() {
        let script = Script::load_unchecked(r#"
        function run()
            now = datetime()
            print(now)
            if regex_find("^\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}$", now) == nil then
                return 'invalid date'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
