use crate::engine::ctx::State;
use crate::hlua;
use std::sync::Arc;


pub fn getopt(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("getopt", hlua::function1(move |key: String| -> Option<String> {
        state.getopt(&key)
            .map(|x| x.to_owned())
    }))
}


#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_getopt() {
        let script = Script::load_unchecked(r#"
        function run()
            x = getopt('hello')
            if x ~= nil then
                return 'Weird result'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
