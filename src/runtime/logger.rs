use engine::ctx::State;
use hlua;
use std::sync::Arc;


pub fn info(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("info", hlua::function1(move |msg: String| {
        state.info(msg);
    }))
}
