use crate::engine::ctx::State;
use crate::hlua;
use std::sync::Arc;
use std::thread;
use std::time::Duration;


pub fn sleep(lua: &mut hlua::Lua, _: Arc<dyn State>) {
    lua.set("sleep", hlua::function1(move |n: i32| {
        thread::sleep(Duration::from_secs(n as u64));
    }))
}
