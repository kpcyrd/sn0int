use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use crate::xml;
use std::sync::Arc;


pub fn xml_decode(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("xml_decode", hlua::function1(move |x: String| -> Result<AnyLuaValue> {
        xml::decode(&x)
            .map_err(|err| state.set_error(err))
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
            if last_err() then return end
            print(x)

            body = x['children'][1]
            if body['name'] ~= 'body' then
                return 'wrong body tag name'
            end

            foo = body['named']['foo']
            if foo['name'] ~= 'foo' then
                return 'foo has wrong tag name'
            end
            if foo['attrs']['fizz'] ~= 'buzz' then
                return 'foo has wrong fizz attribute'
            end
            if foo['text'] ~= 'bar' then
                return 'foo has wrong inner text'
            end
            if foo['children'][1] then
                return 'foo has unexpected children'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
