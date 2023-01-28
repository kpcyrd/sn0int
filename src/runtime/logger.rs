use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use std::fmt::Write;
use std::sync::Arc;

pub fn format_lua(out: &mut String, x: &AnyLuaValue) -> Result<()> {
    match *x {
        AnyLuaValue::LuaNil => out.push_str("null"),
        AnyLuaValue::LuaString(ref x) => write!(out, "{:?}", x)?,
        AnyLuaValue::LuaNumber(ref x) => write!(out, "{:?}", x)?,
        AnyLuaValue::LuaAnyString(ref x) => write!(out, "{:?}", x.0)?,
        AnyLuaValue::LuaBoolean(ref x) => write!(out, "{:?}", x)?,
        AnyLuaValue::LuaArray(ref x) => {
            out.push('{');
            let mut first = true;

            for (k, v) in x {
                if !first {
                    out.push_str(", ");
                }

                let mut key = String::new();
                format_lua(&mut key, k)?;

                let mut value = String::new();
                format_lua(&mut value, v)?;

                write!(out, "{}: {}", key, value)?;

                first = false;
            }
            out.push('}');
        },
        AnyLuaValue::LuaOther => out.push_str("LuaOther"),
    }

    Ok(())
}

pub fn info(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("info", hlua::function1(move |val: AnyLuaValue| {
        let mut out = String::new();
        format_lua(&mut out, &val).expect("out of memory");
        state.info(out);
    }))
}

pub fn debug(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("debug", hlua::function1(move |val: AnyLuaValue| {
        let mut out = String::new();
        format_lua(&mut out, &val).expect("out of memory");
        state.debug(out);
    }))
}

pub fn error(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("error", hlua::function1(move |msg: String| {
        state.error(msg);
    }))
}

pub fn warn(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("warn", hlua::function1(move |msg: String| {
        state.warn(msg);
    }))
}

pub fn warn_once(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("warn_once", hlua::function1(move |msg: String| {
        state.warn_once(msg);
    }))
}

pub fn status(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("status", hlua::function1(move |msg: String| {
        state.status(msg);
    }))
}

pub fn print(lua: &mut hlua::Lua, _: Arc<dyn State>) {
    lua.set("print", hlua::function1(move |val: AnyLuaValue| {
        // println!("{:?}", val);
        let mut out = String::new();
        format_lua(&mut out, &val).expect("out of memory");
        eprintln!("{}", out);
    }))
}
