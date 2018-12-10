use crate::errors::*;

use crate::engine::ctx::State;
use crate::hlua;
use std::sync::Arc;


pub fn stdin_readline(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("stdin_readline", hlua::function0(move || -> Result<Option<String>> {
        state.stdin_readline()
            .map_err(|e| state.set_error(e))
    }))
}
