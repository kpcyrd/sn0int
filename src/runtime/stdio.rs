use crate::errors::*;

use crate::engine::ctx::State;
use crate::hlua;
use std::sync::Arc;


pub fn stdin_read_line(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    let s = state;
    let state = s.clone();
    lua.set("stdin_read_line", hlua::function0(move || -> Result<Option<String>> {
        state.stdin_read_line()
            .map_err(|e| state.set_error(e))
    }));
    // TODO: deprecate stdin_readline
    let state = s.clone();
    lua.set("stdin_readline", hlua::function0(move || -> Result<Option<String>> {
        state.stdin_read_line()
            .map_err(|e| state.set_error(e))
    }));
}

pub fn stdin_read_to_end(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("stdin_read_to_end", hlua::function0(move || -> Result<Option<String>> {
        state.stdin_read_to_end()
            .map_err(|e| state.set_error(e))
    }))
}
