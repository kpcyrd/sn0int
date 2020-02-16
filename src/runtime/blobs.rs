use crate::errors::*;
use crate::engine::ctx::State;
use crate::engine::structs::byte_array;
use crate::hlua::{self, AnyLuaValue};
use sn0int_std::blobs::{Blob, BlobState};
use std::sync::Arc;


pub fn create_blob<S>(lua: &mut hlua::Lua, state: Arc<S>)
    where S: State + BlobState + 'static
{
    lua.set("create_blob", hlua::function1(move |bytes: AnyLuaValue| -> Result<String> {
        let bytes = byte_array(bytes)
            .map_err(|err| state.set_error(err))?;

        let blob = Blob::create(bytes.into());
        let id = state.register_blob(blob);

        Ok(id)
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_create_blob() {
        let script = Script::load_unchecked(r#"
        function run()
            blob = create_blob("asdf")
            if blob ~= "DTTV3EjpHBNJx3Zw7eJsVPm4bYXKmNkJQpVNkcvTtTSz" then
                return 'unexpected blob: ' .. blob
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
