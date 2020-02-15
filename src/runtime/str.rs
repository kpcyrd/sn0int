use crate::engine::ctx::State;
use crate::hlua;
use std::sync::Arc;


pub fn str_find(lua: &mut hlua::Lua, _state: Arc<dyn State>) {
    lua.set("str_find", hlua::function2(move |s: String, needle: String| -> Option<u32> {
        s.find(&needle)
            .map(|i| i as u32 + 1)
    }))
}

pub fn str_replace(lua: &mut hlua::Lua, _state: Arc<dyn State>) {
    lua.set("str_replace", hlua::function3(move |s: String, search: String, replace: String| -> String {
        s.replace(&search, &replace)
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_str_find() {
        let script = Script::load_unchecked(r#"
        function run()
            x = str_find('asdf', 'asdf')
            if x ~= 1 then return 'error 1' end
            x = str_find('asdf', 'a')
            if x ~= 1 then return 'error 2' end
            x = str_find('asdf', 's')
            if x ~= 2 then return 'error 3' end
            x = str_find('asdf', 'x')
            if x ~= nil then return 'error 4' end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_str_replace() {
        let script = Script::load_unchecked(r#"
        function run()
            x = str_replace('asdf', 'asdf', '')
            if x ~= '' then return 'error 1: ' .. x end

            x = str_replace('asdf', '', 'x')
            if x ~= 'xaxsxdxfx' then return 'error 2: ' .. x end

            x = str_replace('asdf', 's', 'x')
            if x ~= 'axdf' then return 'error 3: ' .. x end

            x = str_replace('asdf', 'sd', 'xxx')
            if x ~= 'axxxf' then return 'error 4: ' .. x end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
