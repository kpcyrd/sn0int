use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;

pub fn intval(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("intval", hlua::function1(move |val: AnyLuaValue| -> Result<AnyLuaValue> {
        match val {
            AnyLuaValue::LuaNil => Ok(AnyLuaValue::LuaNil),
            AnyLuaValue::LuaString(ref x) => {
                let n = x.parse::<i64>()
                    .map_err(|e| state.set_error(Error::from(e)))?;
                Ok(AnyLuaValue::LuaNumber(n as f64))
            },
            AnyLuaValue::LuaNumber(ref x) => {
                Ok(AnyLuaValue::LuaNumber(x.floor()))
            },
            _ => Err(state.set_error(format_err!("Unsupported input for intval"))),
        }
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_intval_int() {
        let script = Script::load_unchecked(r#"
        function run()
            x = intval(123)
            if x ~= 123 then
                return 'wrong ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_intval_str() {
        let script = Script::load_unchecked(r#"
        function run()
            x = intval('123')
            if x ~= 123 then
                return 'wrong ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_intval_float_1() {
        let script = Script::load_unchecked(r#"
        function run()
            x = intval(0.456)
            if x ~= 0 then
                return 'wrong ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_intval_float_2() {
        let script = Script::load_unchecked(r#"
        function run()
            x = intval(123.456)
            if x ~= 123 then
                return 'wrong ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
