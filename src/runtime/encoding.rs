use crate::errors::*;

use crate::engine::ctx::State;
use crate::engine::structs::{byte_array, lua_bytes};
use crate::hlua::{self, AnyLuaValue};
use data_encoding::{BASE64, Specification, Encoding};
use std::sync::Arc;


fn spec(symbols: &str, padding: &str) -> Result<Encoding> {
    let mut spec = Specification::new();
    spec.symbols.push_str(symbols);
    if let Some(padding) = padding.chars().next() {
        spec.padding = Some(padding);
    }
    spec.encoding()
        .map_err(Error::from)
}

pub fn base64_decode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("base64_decode", hlua::function1(move |bytes: String| -> Result<AnyLuaValue> {
        BASE64.decode(bytes.as_bytes())
            .map_err(|err| state.set_error(err.into()))
            .map(|bytes| lua_bytes(&bytes))
    }))
}

pub fn base64_encode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("base64_encode", hlua::function1(move |bytes: AnyLuaValue| -> Result<String> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| BASE64.encode(&bytes))
    }))
}

pub fn base64_custom_decode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("base64_custom_decode", hlua::function3(move |bytes: String, alphabet: String, padding: String| -> Result<AnyLuaValue> {
        if alphabet.len() != 64 {
            bail!("alphabet isn't base64");
        }
        let spec = spec(&alphabet, &padding)
            .map_err(|err| state.set_error(err))?;
        spec.decode(bytes.as_bytes())
            .map_err(|err| state.set_error(err.into()))
            .map(|bytes| lua_bytes(&bytes))
    }))
}

pub fn base64_custom_encode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("base64_custom_encode", hlua::function3(move |bytes: AnyLuaValue, alphabet: String, padding: String| -> Result<String> {
        if alphabet.len() != 64 {
            bail!("alphabet isn't base64");
        }
        let spec = spec(&alphabet, &padding)
            .map_err(|err| state.set_error(err))?;
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| spec.encode(&bytes))
    }))
}

pub fn base32_custom_decode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("base32_custom_decode", hlua::function3(move |bytes: String, alphabet: String, padding: String| -> Result<AnyLuaValue> {
        if alphabet.len() != 32 {
            bail!("alphabet isn't base32");
        }
        let spec = spec(&alphabet, &padding)
            .map_err(|err| state.set_error(err))?;
        spec.decode(bytes.as_bytes())
            .map_err(|err| state.set_error(err.into()))
            .map(|bytes| lua_bytes(&bytes))
    }))
}

pub fn base32_custom_encode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("base32_custom_encode", hlua::function3(move |bytes: AnyLuaValue, alphabet: String, padding: String| -> Result<String> {
        if alphabet.len() != 32 {
            bail!("alphabet isn't base32");
        }
        let spec = spec(&alphabet, &padding)
            .map_err(|err| state.set_error(err))?;
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| spec.encode(&bytes))
    }))
}


#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_base64_encode() {
        // ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/
        // =
        let script = Script::load_unchecked(r#"
        function run()
            x = base64_encode('ohai')
            if x ~= 'b2hhaQ==' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base64_decode() {
        // ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/
        // =
        let script = Script::load_unchecked(r#"
        function run()
            x = base64_decode('b2hhaQ==')
            if x ~= 'ohai' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base64_custom_encode() {
        // ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/
        // =
        let script = Script::load_unchecked(r#"
        function run()
            x = base64_custom_encode('ohai', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/', '=')
            if x ~= 'b2hhaQ==' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base64_custom_decode() {
        // ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/
        // =
        let script = Script::load_unchecked(r#"
        function run()
            x = base64_custom_decode('b2hhaQ==', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/', '=')
            if x ~= 'ohai' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base64_custom_unpadded_encode() {
        // ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/
        let script = Script::load_unchecked(r#"
        function run()
            x = base64_custom_encode('ohai', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/', '')
            if x ~= 'b2hhaQ' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base64_custom_unpadded_decode() {
        // ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/
        let script = Script::load_unchecked(r#"
        function run()
            x = base64_custom_decode('b2hhaQ', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/', '')
            if x ~= 'ohai' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base64_custom_urlsafe_encode() {
        // ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_
        // =
        let script = Script::load_unchecked(r#"
        function run()
            x = base64_custom_encode('ohai', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_', '=')
            if x ~= 'b2hhaQ==' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base64_custom_urlsafe_decode() {
        // ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_
        // =
        let script = Script::load_unchecked(r#"
        function run()
            x = base64_custom_decode('b2hhaQ==', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_', '=')
            if x ~= 'ohai' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base32_custom_encode() {
        // ABCDEFGHIJKLMNOPQRSTUVWXYZ234567
        // =
        let script = Script::load_unchecked(r#"
        function run()
            x = base32_custom_encode('ohai', 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567', '=')
            if x ~= 'N5UGC2I=' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base32_custom_decode() {
        // ABCDEFGHIJKLMNOPQRSTUVWXYZ234567
        // =
        let script = Script::load_unchecked(r#"
        function run()
            x = base32_custom_decode('N5UGC2I=', 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567', '=')
            if x ~= 'ohai' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base32_custom_zbase32_encode() {
        // ybndrfg8ejkmcpqxot1uwisza345h769
        let script = Script::load_unchecked(r#"
        function run()
            x = base32_custom_encode('ohai', 'ybndrfg8ejkmcpqxot1uwisza345h769', '')
            if x ~= 'p7wgn4e' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_base32_custom_zbase32_decode() {
        // ybndrfg8ejkmcpqxot1uwisza345h769
        let script = Script::load_unchecked(r#"
        function run()
            x = base32_custom_decode('p7wgn4e', 'ybndrfg8ejkmcpqxot1uwisza345h769', '')
            if x ~= 'ohai' then
                return 'encode failed: ' .. x
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
