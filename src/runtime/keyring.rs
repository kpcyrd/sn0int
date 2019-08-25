use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;


pub fn keyring(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("keyring", hlua::function1(move |query: String| -> Vec<AnyLuaValue> {
        state.keyring(&query).into_iter()
            .map(|x| x.to_lua().unwrap())
            .collect()
    }))
}
