use crate::errors::*;

use crate::engine::ctx::State;
use crate::engine::structs::byte_array;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;


pub fn hex(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("hex", hlua::function1(move |bytes: AnyLuaValue| -> Result<String> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| {
                let mut out = String::new();

                for b in bytes {
                    out += &format!("{:02x}", b);
                }

                out
            })
    }))
}
