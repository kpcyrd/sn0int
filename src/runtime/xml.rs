use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use crate::json::LuaJsonValue;
use serde_json::Value;
use std::sync::Arc;


pub fn xml_decode(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("xml_decode", hlua::function1(move |xml: String| -> Result<AnyLuaValue> {
        let v: Value = serde_xml_rs::from_str(&xml)
            .map_err(|err| state.set_error(format_err!("{}", err)))?;
        let v: LuaJsonValue = v.into();
        Ok(v.into())
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_xml_decode() {
        let script = Script::load_unchecked(r#"
        function run()
            x = xml_decode('<body><foo fizz="buzz">bar</foo></body>')
            print(x)
            if x['foo']['fizz'] ~= 'buzz' then
                return 'wrong xml value for <foo fizz="???">'
            end
            if x['foo']['$value'] ~= 'bar' then
                return 'wrong xml value for <foo>???</foo>'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
