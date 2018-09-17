use errors::*;

use engine::ctx::State;
use hlua::{self, AnyLuaValue};
use std::sync::Arc;
use hlua::AnyHashableLuaValue;
use std::collections::HashMap;
use web::{RequestOptions, HttpRequest};
use errors::{Result, Error};


pub fn http_mksession(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("http_mksession", hlua::function0(move || -> String {
        state.http_mksession()
    }))
}

pub fn http_request(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("http_request", hlua::function4(move |session: String, method: String, url: String, options: AnyLuaValue| -> Result<AnyLuaValue> {
        RequestOptions::try_from(options)
            .context("invalid request options")
            .map_err(|err| state.set_error(Error::from(err)))
            .map(|options| {
                state.http_request(&session, method, url, options).into()
            })
    }))
}

pub fn http_send(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("http_send", hlua::function1(move |request: AnyLuaValue| -> Result<HashMap<AnyHashableLuaValue, AnyLuaValue>> {
        let req = match HttpRequest::try_from(request)
                                .context("invalid http request object") {
            Ok(req) => req,
            Err(err) => return Err(state.set_error(Error::from(err))),
        };

        req.send(&state)
            .map_err(|err| state.set_error(err))
            .map(|resp| resp.into())
    }))
}
