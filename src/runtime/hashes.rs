use crate::errors::*;
use crate::engine::ctx::State;
use crate::engine::structs::{byte_array, lua_bytes};
use crate::hlua::{self, AnyLuaValue};
use digest::Digest;
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use std::sync::Arc;


pub fn md5(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("md5", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&Md5::digest(&bytes)))
    }))
}

pub fn sha1(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sha1", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&Sha1::digest(&bytes)))
    }))
}

pub fn sha2_256(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sha2_256", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&Sha256::digest(&bytes)))
    }))
}

pub fn sha2_512(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sha2_512", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&Sha512::digest(&bytes)))
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_md5() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(md5("abcdef"))
            print(h)
            if h ~= 'e80b5017098950fc58aad83c8c14978e' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_sha1() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(sha1("abcdef"))
            print(h)
            if h ~= '1f8ac10f23c5b5bc1167bda84b833e5c057a77d2' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_sha2_256() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(sha2_256("abcdef"))
            print(h)
            if h ~= 'bef57ec7f53a6d40beb640a780a639c83bc29ac8a9816f1fc6c5c6dcd93c4721' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_sha2_512() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(sha2_512("abcdef"))
            print(h)
            if h ~= 'e32ef19623e8ed9d267f657a81944b3d07adbb768518068e88435745564e8d4150a0a703be2a7d88b61e3d390c2bb97e2d4c311fdc69d6b1267f05f59aa920e7' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
