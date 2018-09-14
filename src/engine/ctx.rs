use errors::*;

use hlua;
use runtime;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct State {
    error: Arc<Mutex<Option<Error>>>,
}

impl State {
    pub fn new() -> State {
        State {
            error: Arc::new(Mutex::new(None)),
        }
    }

    pub fn last_error(&self) -> Option<String> {
        let lock = self.error.lock().unwrap();
        lock.as_ref().map(|err| err.to_string())
    }

    pub fn set_error(&self, err: Error) -> Error {
        let mut mtx = self.error.lock().unwrap();
        let cp = format_err!("{:?}", err);
        *mtx = Some(err);
        cp.into()
    }
}

#[derive(Debug, Clone)]
pub struct Script {
    code: String,
}

#[allow(dead_code)]
fn ctx<'a>() -> (hlua::Lua<'a>, Arc<State>) {
    let mut lua = hlua::Lua::new();
    lua.open_string();
    let state = Arc::new(State::new());

    // runtime::html_form(&mut lua, state.clone());
    // runtime::html_select(&mut lua, state.clone());
    // runtime::html_select_list(&mut lua, state.clone());
    // runtime::http_mksession(&mut lua, state.clone());
    // runtime::http_request(&mut lua, state.clone());
    // runtime::http_send(&mut lua, state.clone());
    // runtime::json_decode(&mut lua, state.clone());
    // runtime::json_encode(&mut lua, state.clone());
    // runtime::last_err(&mut lua, state.clone());
    runtime::print(&mut lua, state.clone());
    runtime::url_join(&mut lua, state.clone());
    runtime::url_parse(&mut lua, state.clone());

    (lua, state)
}

impl Script {
    pub fn load_unchecked(code: String) -> Result<Script> {
        /*
        let (mut lua, _) = ctx();

        // TODO: we do not want to execute the script outside of the sandbox
        lua.execute::<()>(&code)?;

        let descr = {
            let descr: hlua::StringInLua<_> = lua.get("descr")
                .ok_or_else(|| format_err!("descr undefined"))?;
            (*descr).to_owned()
        };
        */

        Ok(Script {
            code,
        })
    }
}
