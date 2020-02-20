use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;
use crate::json;


pub fn str_find(lua: &mut hlua::Lua, _state: Arc<dyn State>) {
    lua.set("str_find", hlua::function2(move |s: String, needle: String| -> Option<u32> {
        s.find(&needle)
            .map(|i| i as u32 + 1)
    }))
}

pub fn str_replace(lua: &mut hlua::Lua, _state: Arc<dyn State>) {
    lua.set("str_replace", hlua::function3(move |s: String, search: String, replace: String| -> String {
        s.replace(&search, &replace)
    }))
}

pub fn strval(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("strval", hlua::function1(move |x: AnyLuaValue| -> Result<AnyLuaValue> {
        let x = match x {
            AnyLuaValue::LuaNil => x,
            AnyLuaValue::LuaBoolean(v) => if v {
                AnyLuaValue::LuaString("true".into())
            } else {
                AnyLuaValue::LuaString("false".into())
            },
            AnyLuaValue::LuaString(_) => x,
            AnyLuaValue::LuaNumber(v) => {
                // this is needed or every number is detected as float
                let v = if v % 1f64 == 0f64 {
                    (v as u64).to_string()
                } else {
                    v.to_string()
                };
                AnyLuaValue::LuaString(v)
            },
            AnyLuaValue::LuaAnyString(_) => x,
            AnyLuaValue::LuaArray(_) => {
                let v = json::encode(x)
                    .map_err(|err| state.set_error(err))?;
                AnyLuaValue::LuaString(v)
            },
            AnyLuaValue::LuaOther => x,
        };
        Ok(x)
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_str_find() {
        let script = Script::load_unchecked(r#"
        function run()
            x = str_find('asdf', 'asdf')
            if x ~= 1 then return 'error 1' end
            x = str_find('asdf', 'a')
            if x ~= 1 then return 'error 2' end
            x = str_find('asdf', 's')
            if x ~= 2 then return 'error 3' end
            x = str_find('asdf', 'x')
            if x ~= nil then return 'error 4' end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_str_replace() {
        let script = Script::load_unchecked(r#"
        function run()
            x = str_replace('asdf', 'asdf', '')
            if x ~= '' then return 'error 1: ' .. x end

            x = str_replace('asdf', '', 'x')
            if x ~= 'xaxsxdxfx' then return 'error 2: ' .. x end

            x = str_replace('asdf', 's', 'x')
            if x ~= 'axdf' then return 'error 3: ' .. x end

            x = str_replace('asdf', 'sd', 'xxx')
            if x ~= 'axxxf' then return 'error 4: ' .. x end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_strval() {
        let script = Script::load_unchecked(r#"
        function run()
            x = strval("asdf")
            if x ~= "asdf" then return 'error 1: ' .. x end

            x = strval(1234)
            if x ~= "1234" then return 'error 2: ' .. x end

            x = strval(313.37)
            if x ~= "313.37" then return 'error 3: ' .. x end

            x = strval(true)
            if x ~= "true" then return 'error 4: ' .. x end

            x = strval(false)
            if x ~= "false" then return 'error 5: ' .. x end

            x = strval(nil)
            if x ~= nil then return 'error 6: ' .. x end

            x = strval({a=1})
            if x ~= '{"a":1}' then return 'error 7: ' .. x end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
