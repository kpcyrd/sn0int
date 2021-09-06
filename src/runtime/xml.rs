use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use crate::xml;
use std::sync::Arc;


pub fn xml_decode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("xml_decode", hlua::function1(move |x: String| -> Result<AnyLuaValue> {
        xml::decode(&x)
            .map_err(|err| state.set_error(err))
    }))
}

#[inline]
fn key_is(key: &AnyLuaValue, expected: &str) -> bool {
    match key {
        AnyLuaValue::LuaString(key) => key.as_str() == expected,
        _ => false,
    }
}

#[inline]
fn get_children(xml: AnyLuaValue) -> Option<AnyLuaValue> {
    match xml {
        AnyLuaValue::LuaArray(arr) => {
            for (key, value) in arr {
                if key_is(&key, "children") {
                    return Some(value);
                }
            }

            None
        }
        _ => None,
    }
}

#[inline]
fn match_element_name(xml: &AnyLuaValue, name: &str) -> bool {
    match xml {
        AnyLuaValue::LuaArray(arr) => {
            for (key, value) in arr {
                if key_is(key, "name") {
                    match value {
                        AnyLuaValue::LuaString(key) => {
                            return key.as_str() == name
                        },
                        _ => return false,
                    }
                }
            }

            false
        },
        _ => false,
    }
}

pub fn xml_named(lua: &mut hlua::Lua, _state: Arc<dyn State>) {
    lua.set("xml_named", hlua::function2(move |xml: AnyLuaValue, name: String| -> AnyLuaValue {
        if let Some(AnyLuaValue::LuaArray(arr)) = get_children(xml) {
            for (_, value) in arr {
                if match_element_name(&value, &name) {
                    return value;
                }
            }
        }

        AnyLuaValue::LuaNil
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

            foo = xml_named(body, 'foo')
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
