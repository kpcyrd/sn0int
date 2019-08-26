use crate::errors::*;

use crate::engine::ctx::State;
use crate::engine::structs::LuaList;
use crate::hlua::{self, AnyLuaValue};
use regex::{Regex, Captures};
use std::sync::Arc;


fn capture_to_lua(caps: &Captures) -> AnyLuaValue {
    let mut i = 0;
    let mut list = LuaList::new();
    while let Some(cap) = caps.get(i) {
        list.push_str(cap.as_str());
        i += 1;
    }
    list.into()
}

pub fn regex_find(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("regex_find", hlua::function2(move |regex: String, data: String| -> Result<AnyLuaValue> {
        let re = Regex::new(&regex)
            .map_err(|err| state.set_error(Error::from(err)))?;

        match re.captures(&data) {
            Some(caps) => Ok(capture_to_lua(&caps)),
            None => Ok(AnyLuaValue::LuaNil),
        }
    }))
}

pub fn regex_find_all(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("regex_find_all", hlua::function2(move |regex: String, data: String| -> Result<Vec<AnyLuaValue>> {
        let re = Regex::new(&regex)
            .map_err(|err| state.set_error(Error::from(err)))?;

        let list = re.captures_iter(&data)
            .map(|x| capture_to_lua(&x))
            .collect();

        Ok(list)
    }))
}


#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_find() {
        let script = Script::load_unchecked(r#"
        function run()
            m = regex_find(".(.)", "abcdef")

            if m == nil then
                return 'No captures'
            end

            if m[1] ~= 'ab' then
                return 'Incorrect result in m[1]'
            end

            if m[2] ~= 'b' then
                return 'Incorrect result in m[2]'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_find_all() {
        let script = Script::load_unchecked(r#"
        function run()
            m = regex_find_all(".(.)", "abcdef")

            if m[1][1] ~= 'ab' then
                return 'Incorrect result in m[1][1]'
            end

            if m[1][2] ~= 'b' then
                return 'Incorrect result in m[1][2]'
            end

            if m[2][1] ~= 'cd' then
                return 'Incorrect result in m[2][1]'
            end

            if m[2][2] ~= 'd' then
                return 'Incorrect result in m[2][2]'
            end

            if m[3][1] ~= 'ef' then
                return 'Incorrect result in m[3][1]'
            end

            if m[3][2] ~= 'f' then
                return 'Incorrect result in m[3][2]'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
