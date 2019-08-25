use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;


pub fn clear_err(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("clear_err", hlua::function0(move || {
        state.clear_error()
    }))
}

pub fn last_err(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("last_err", hlua::function0(move || -> AnyLuaValue {
        match state.last_error() {
            Some(err) => AnyLuaValue::LuaString(err),
            None => AnyLuaValue::LuaNil,
        }
    }))
}

pub fn set_err(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("set_err", hlua::function1(move |err: String| {
        state.set_error(failure::err_msg(err));
    }))
}
