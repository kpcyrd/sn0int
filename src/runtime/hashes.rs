use crate::errors::*;
use crate::engine::ctx::State;
use crate::engine::structs::{byte_array, lua_bytes};
use crate::hlua::{self, AnyLuaValue};
use digest::{Digest, Update, BlockInput, FixedOutput, Reset};
use digest::generic_array::ArrayLength;
use hmac::{Hmac, Mac, NewMac};
use std::sync::Arc;


pub fn md5(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("md5", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&md5::Md5::digest(&bytes)))
    }))
}

pub fn sha1(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sha1", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&sha1::Sha1::digest(&bytes)))
    }))
}

pub fn sha2_256(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sha2_256", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&sha2::Sha256::digest(&bytes)))
    }))
}

pub fn sha2_512(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sha2_512", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&sha2::Sha512::digest(&bytes)))
    }))
}

pub fn sha3_256(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sha3_256", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&sha3::Sha3_256::digest(&bytes)))
    }))
}

pub fn sha3_512(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sha3_512", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&sha3::Sha3_512::digest(&bytes)))
    }))
}

fn hmac<D>(secret: AnyLuaValue, msg: AnyLuaValue) -> Result<AnyLuaValue>
    where
        D: Update + BlockInput + FixedOutput + Reset + Default + Clone,
        D::BlockSize: ArrayLength<u8> + Clone,
        D::OutputSize: ArrayLength<u8>,
{
    let secret = byte_array(secret)?;
    let msg = byte_array(msg)?;

    let mut mac = match Hmac::<D>::new_varkey(&secret) {
        Ok(mac) => mac,
        Err(_) => bail!("Invalid key length"),
    };
    mac.update(&msg);
    let result = mac.finalize();
    Ok(lua_bytes(&result.into_bytes()))
}

pub fn hmac_md5(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("hmac_md5", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<md5::Md5>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn hmac_sha1(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("hmac_sha1", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<sha1::Sha1>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn hmac_sha2_256(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("hmac_sha2_256", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<sha2::Sha256>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn hmac_sha2_512(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("hmac_sha2_512", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<sha2::Sha512>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn hmac_sha3_256(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("hmac_sha3_256", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<sha3::Sha3_256>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn hmac_sha3_512(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("hmac_sha3_512", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<sha3::Sha3_512>(secret, msg)
            .map_err(|err| state.set_error(err))
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

    #[test]
    fn verify_sha3_256() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(sha3_256("abcdef"))
            print(h)
            if h ~= '59890c1d183aa279505750422e6384ccb1499c793872d6f31bb3bcaa4bc9f5a5' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_sha3_512() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(sha3_512("abcdef"))
            print(h)
            if h ~= '01309a45c57cd7faef9ee6bb95fed29e5e2e0312af12a95fffeee340e5e5948b4652d26ae4b75976a53cc1612141af6e24df36517a61f46a1a05f59cf667046a' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_hmac_md5() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(hmac_md5("foo", "bar"))
            print(h)
            if h ~= '0c7a250281315ab863549f66cd8a3a53' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_hmac_sha1() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(hmac_sha1("foo", "bar"))
            print(h)
            if h ~= '46b4ec586117154dacd49d664e5d63fdc88efb51' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_hmac_sha2_256() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(hmac_sha2_256("foo", "bar"))
            print(h)
            if h ~= 'f9320baf0249169e73850cd6156ded0106e2bb6ad8cab01b7bbbebe6d1065317' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_hmac_sha2_512() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(hmac_sha2_512("foo", "bar"))
            print(h)
            if h ~= '114682914c5d017dfe59fdc804118b56a3a652a0b8870759cf9e792ed7426b08197076bf7d01640b1b0684df79e4b67e37485669e8ce98dbab60445f0db94fce' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_hmac_sha3_256() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(hmac_sha3_256("foo", "bar"))
            print(h)
            if h ~= 'a7dc3fbbd45078239f0cb321e6902375d22b505f2c48722eb7009e7da2574893' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_hmac_sha3_512() {
        let script = Script::load_unchecked(r#"
        function run()
            h = hex(hmac_sha3_512("foo", "bar"))
            print(h)
            if h ~= '2da91b8227d106199fd06c5d8a6752796cf3c84dde5a427bd2aca384f0cffc19997e2584ed15c55542c2cb8918b987e2bcd9e77a9f3fdbb4dbea8a3d0136da2f' then
                return 'incorrect hash'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
