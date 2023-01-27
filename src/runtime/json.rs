use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;
use crate::json;


pub fn json_decode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("json_decode", hlua::function1(move |x: String| -> Result<AnyLuaValue> {
        json::decode(x)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn json_encode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("json_encode", hlua::function1(move |x: AnyLuaValue| -> Result<String> {
        json::encode(x)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn json_decode_stream(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("json_decode_stream", hlua::function1(move |x: String| -> Result<Vec<AnyLuaValue>> {
        json::decode_stream(&x)
            .map_err(|err| state.set_error(err))
    }))
}


#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_json_encode() {
        let script = Script::load_unchecked(r#"
        function run()
            x = json_encode({1,2,3,4})
            if x ~= '[1,2,3,4]' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_json_decode() {
        let script = Script::load_unchecked(r#"
        function run()
            x = json_decode('[1,2,3,4]')
            if not (x[1] == 1 and x[2] == 2 and x[3] == 3 and x[4] == 4) then
                return 'decode failed: ' .. json_encode(x)
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_json_decode_stream() {
        let script = Script::load_unchecked(r#"
        function run()
            x = json_decode_stream('[1,2][3,4]')
            if not (x[1][1] == 1 and x[1][2] == 2 and x[2][1] == 3 and x[2][2] == 4) then
                return 'decode_stream failed: ' .. json_encode(x)
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
