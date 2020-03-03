use crate::errors::*;

use crate::engine::ctx::State;
use crate::engine::structs::{byte_array, lua_bytes};
use crate::hlua::{self, AnyLuaValue};
use sn0int_std::crypto;
use std::sync::Arc;

pub fn key_trunc_pad(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("key_trunc_pad", hlua::function3(move |bytes: AnyLuaValue, len: u32, pad: u8| -> Result<AnyLuaValue> {
        let bytes = byte_array(bytes)
            .map_err(|err| state.set_error(err))?;
        let bytes = crypto::key_trunc_pad(&bytes, len as usize, pad);
        Ok(lua_bytes(&bytes))
    }))
}

pub fn sodium_secretbox_open(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sodium_secretbox_open", hlua::function2(move |encrypted: AnyLuaValue, key: AnyLuaValue| -> Result<AnyLuaValue> {
        let encrypted = byte_array(encrypted)
            .map_err(|err| state.set_error(err))?;
        let key = byte_array(key)
            .map_err(|err| state.set_error(err))?;

        let plain = crypto::sodium_secretbox_open(&encrypted, &key)
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&plain))
    }))
}
