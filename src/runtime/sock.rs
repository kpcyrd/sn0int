use crate::errors::*;

use crate::engine::ctx::State;
use crate::engine::structs::{byte_array, lua_bytes};
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;


pub fn sock_connect(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_connect", hlua::function3(move |host: String, port: u16, _settings: AnyLuaValue| -> Result<String> {
        state.sock_connect(&host, port)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn sock_send(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_send", hlua::function2(move |sock: String, bytes: AnyLuaValue| -> Result<()> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = byte_array(bytes)
            .map_err(|err| state.set_error(err))?;

        sock.send(&bytes)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn sock_recv(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_recv", hlua::function1(move |sock: String| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = sock.recv()
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_sendline(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_sendline", hlua::function2(move |sock: String, line: String| -> Result<()> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        sock.sendline(&line)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn sock_recvline(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_recvline", hlua::function1(move |sock: String| -> Result<String> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let line = sock.recvline()
            .map_err(|err| state.set_error(err))?;

        Ok(line)
    }))
}

pub fn sock_recvall(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_recvall", hlua::function1(move |sock: String| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = sock.recvall()
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_recvline_contains(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_recvline_contains", hlua::function2(move |sock: String, needle: String| -> Result<String> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let line = sock.recvline_contains(&needle)
            .map_err(|err| state.set_error(err))?;

        Ok(line)
    }))
}

pub fn sock_recvline_regex(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_recvline_regex", hlua::function2(move |sock: String, regex: String| -> Result<String> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let line = sock.recvline_regex(&regex)
            .map_err(|err| state.set_error(err))?;

        Ok(line)
    }))
}

pub fn sock_recvn(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_recvn", hlua::function2(move |sock: String, n: u32| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = sock.recvn(n)
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_recvuntil(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_recvuntil", hlua::function2(move |sock: String, delim: AnyLuaValue| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let delim = byte_array(delim)
            .map_err(|err| state.set_error(err))?;

        let bytes = sock.recvuntil(&delim)
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_sendafter(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_sendafter", hlua::function3(move |sock: String, delim: AnyLuaValue, bytes: AnyLuaValue| -> Result<()> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let delim = byte_array(delim)
            .map_err(|err| state.set_error(err))?;

        let bytes = byte_array(bytes)
            .map_err(|err| state.set_error(err))?;

        sock.sendafter(&delim, &bytes)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn sock_newline(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("sock_newline", hlua::function2(move |sock: String, newline: String| -> () {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        sock.newline(newline);
    }))
}
