use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua;
use std::sync::Arc;


pub fn ratelimit_throttle(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("ratelimit_throttle", hlua::function3(move |key: String, passes: u32, time: u32| -> Result<()> {
        state.ratelimit(key, passes, time)
            .map_err(|e| state.set_error(e))
    }))
}
