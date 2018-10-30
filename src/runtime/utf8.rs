use errors::*;

use engine::ctx::State;
use engine::structs::byte_array;
use hlua::{self, AnyLuaValue};
use std::sync::Arc;


pub fn utf8_decode(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("utf8_decode", hlua::function1(move |bytes: AnyLuaValue| -> Result<String> {
        let bytes = byte_array(bytes)
            .map_err(|err| state.set_error(err))?;
        String::from_utf8(bytes)
            .map_err(|err| state.set_error(err.into()))
    }))
}
