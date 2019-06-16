use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;


fn format_lua(out: &mut String, x: &AnyLuaValue) {
    match *x {
        AnyLuaValue::LuaNil => out.push_str("null"),
        AnyLuaValue::LuaString(ref x) => out.push_str(&format!("{:?}", x)),
        AnyLuaValue::LuaNumber(ref x) => out.push_str(&format!("{:?}", x)),
        AnyLuaValue::LuaAnyString(ref x) => out.push_str(&format!("{:?}", x.0)),
        AnyLuaValue::LuaBoolean(ref x) => out.push_str(&format!("{:?}", x)),
        AnyLuaValue::LuaArray(ref x) => {
            out.push_str("{");
            let mut first = true;

            for &(ref k, ref v) in x {
                if !first {
                    out.push_str(", ");
                }

                let mut key = String::new();
                format_lua(&mut key, &k);

                let mut value = String::new();
                format_lua(&mut value, &v);

                out.push_str(&format!("{}: {}", key, value));

                first = false;
            }
            out.push_str("}");
        },
        AnyLuaValue::LuaOther => out.push_str("LuaOther"),
    }
}

pub fn info(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("info", hlua::function1(move |val: AnyLuaValue| {
        let mut out = String::new();
        format_lua(&mut out, &val);
        state.info(out);
    }))
}

pub fn debug(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("debug", hlua::function1(move |val: AnyLuaValue| {
        let mut out = String::new();
        format_lua(&mut out, &val);
        state.debug(out);
    }))
}

pub fn error(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("error", hlua::function1(move |msg: String| {
        state.error(msg);
    }))
}

pub fn warn(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("warn", hlua::function1(move |msg: String| {
        state.warn(msg);
    }))
}

pub fn warn_once(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("warn_once", hlua::function1(move |msg: String| {
        state.warn_once(msg);
    }))
}

pub fn status(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("status", hlua::function1(move |msg: String| {
        state.status(msg);
    }))
}

pub fn print(lua: &mut hlua::Lua, _: Arc<State>) {
    lua.set("print", hlua::function1(move |val: AnyLuaValue| {
        // println!("{:?}", val);
        let mut out = String::new();
        format_lua(&mut out, &val);
        eprintln!("{}", out);
    }))
}
