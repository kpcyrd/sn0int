use crate::errors::*;

use crate::engine::ctx::State;
use crate::engine::structs::{byte_array, lua_bytes};
use crate::hlua::{self, AnyLuaValue};
use crate::websockets::WebSocketOptions;
use std::sync::Arc;
use crate::json;
use url::Url;

pub fn ws_connect(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("ws_connect", hlua::function2(move |url: String, options: AnyLuaValue| -> Result<String> {
        let options = WebSocketOptions::try_from(options)
            .context("Invalid websocket options")
            .map_err(|err| state.set_error(Error::from(err)))?;

        let url = Url::parse(&url)
            .context("Failed to parse url")
            .map_err(|err| state.set_error(Error::from(err)))?;

        state.ws_connect(url, &options)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn ws_options(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("ws_options", hlua::function2(move |sock: String, options: AnyLuaValue| -> Result<()> {
        let options = WebSocketOptions::try_from(options)
            .context("Invalid websocket options")
            .map_err(|err| state.set_error(Error::from(err)))?;

        let sock = state.get_ws(&sock);
        let sock = sock.lock().unwrap();

        sock.options(&options)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn ws_recv_text(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("ws_recv_text", hlua::function1(move |sock: String| -> Result<Option<String>> {
        let sock = state.get_ws(&sock);
        let mut sock = sock.lock().unwrap();

        let text = sock.read_text()
            .map_err(|err| state.set_error(err))?;

        Ok(text)
    }))
}

pub fn ws_recv_binary(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("ws_recv_binary", hlua::function1(move |sock: String| -> Result<Option<AnyLuaValue>> {
        let sock = state.get_ws(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = sock.read_binary()
            .map_err(|err| state.set_error(err))?
            .map(|bytes| lua_bytes(&bytes));

        Ok(bytes)
    }))
}

pub fn ws_recv_json(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("ws_recv_json", hlua::function1(move |sock: String| -> Result<Option<AnyLuaValue>> {
        let sock = state.get_ws(&sock);
        let mut sock = sock.lock().unwrap();

        let json = sock.read_text()
            .map_err(|err| state.set_error(err))?;

        let json = if let Some(json) = json {
            let json = json::decode(json)
                .map_err(|err| state.set_error(err))?;
            Some(json)
        } else {
            None
        };

        Ok(json)
    }))
}

pub fn ws_send_text(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("ws_send_text", hlua::function2(move |sock: String, text: String| -> Result<()> {
        let sock = state.get_ws(&sock);
        let mut sock = sock.lock().unwrap();

        sock.write_text(text)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn ws_send_binary(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("ws_send_binary", hlua::function2(move |sock: String, bytes: AnyLuaValue| -> Result<()> {
        let sock = state.get_ws(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = byte_array(bytes)
            .map_err(|err| state.set_error(err))?;

        sock.write_binary(bytes)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn ws_send_json(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("ws_send_json", hlua::function2(move |sock: String, json: AnyLuaValue| -> Result<()> {
        let sock = state.get_ws(&sock);
        let mut sock = sock.lock().unwrap();

        let json = json::encode(json)
            .map_err(|err| state.set_error(err))?;

        sock.write_text(json)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}
