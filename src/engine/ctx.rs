use errors::*;

use engine::Reporter;
use hlua::{self, AnyLuaValue};
use models::Object;
use runtime;
use serde_json;
use std::collections::HashMap;
use std::result;
use std::sync::{Arc, Mutex};
use web::{HttpSession, HttpRequest, RequestOptions};
use worker::Event;


pub trait State {
    fn last_error(&self) -> Option<String>;

    fn set_error(&self, err: Error) -> Error;

    fn set_logger(&self, tx: Arc<Mutex<Box<Reporter>>>);

    fn send(&self, msg: &Event);

    fn recv(&self) -> Result<serde_json::Value>;

    fn info(&self, msg: String) {
        self.send(&Event::Info(msg))
    }

    fn status(&self, msg: String) {
        self.send(&Event::Status(msg))
    }

    fn db_insert(&self, object: Object) -> Result<i32> {
        self.send(&Event::Object(object));
        let reply = self.recv()?;
        let reply: result::Result<i32, String> = serde_json::from_value(reply)?;

        reply.map_err(|err| format_err!("Failed to add to database: {:?}", err))
    }

    fn http_mksession(&self) -> String;

    fn http_request(&self, session_id: &str, method: String, url: String, options: RequestOptions) -> HttpRequest;

    fn register_in_jar(&self, session: &str, key: String, value: String);
}

#[derive(Debug, Clone, Default)]
pub struct LuaState {
    error: Arc<Mutex<Option<Error>>>,
    logger: Arc<Mutex<Option<Arc<Mutex<Box<Reporter>>>>>>,
    http_sessions: Arc<Mutex<HashMap<String, HttpSession>>>,
}

impl State for LuaState {
    fn last_error(&self) -> Option<String> {
        let lock = self.error.lock().unwrap();
        lock.as_ref().map(|err| err.to_string())
    }

    fn set_error(&self, err: Error) -> Error {
        let mut mtx = self.error.lock().unwrap();
        let cp = format_err!("{:?}", err);
        *mtx = Some(err);
        cp
    }

    fn set_logger(&self, tx: Arc<Mutex<Box<Reporter>>>) {
        let mut mtx = self.logger.lock().unwrap();
        *mtx = Some(tx);
    }

    fn send(&self, msg: &Event) {
        let mtx = self.logger.lock().unwrap();
        if let Some(mtx) = &*mtx {
            let mut tx = mtx.lock().unwrap();
            tx.send(msg).expect("Failed to write event");
        }
    }

    fn recv(&self) -> Result<serde_json::Value> {
        let mtx = self.logger.lock().unwrap();
        if let Some(mtx) = &*mtx {
            let mut tx = mtx.lock().unwrap();
            tx.recv()
        } else {
            bail!("Failed to read from reporter, non available");
        }
    }

    fn http_mksession(&self) -> String {
        let mut mtx = self.http_sessions.lock().unwrap();
        let (id, session) = HttpSession::new();
        mtx.insert(id.clone(), session);
        id
    }

    fn http_request(&self, session_id: &str, method: String, url: String, options: RequestOptions) -> HttpRequest {
        let mtx = self.http_sessions.lock().unwrap();
        let session = mtx.get(session_id).expect("invalid session reference"); // TODO

        HttpRequest::new(&session, method, url, options)
    }

    fn register_in_jar(&self, session: &str, key: String, value: String) {
        let mut mtx = self.http_sessions.lock().unwrap();
        if let Some(session) = mtx.get_mut(session) {
            session.cookies.register_in_jar(key, value);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    code: String,
}

fn ctx<'a>() -> (hlua::Lua<'a>, Arc<LuaState>) {
    debug!("Creating lua context");
    let mut lua = hlua::Lua::new();
    lua.open_string();
    let state = Arc::new(LuaState::default());

    runtime::db_add(&mut lua, state.clone());
    runtime::dns(&mut lua, state.clone());
    runtime::html_select(&mut lua, state.clone());
    runtime::html_select_list(&mut lua, state.clone());
    runtime::http_mksession(&mut lua, state.clone());
    runtime::http_request(&mut lua, state.clone());
    runtime::http_send(&mut lua, state.clone());
    runtime::info(&mut lua, state.clone());
    runtime::json_decode(&mut lua, state.clone());
    runtime::json_encode(&mut lua, state.clone());
    runtime::json_decode_stream(&mut lua, state.clone());
    runtime::last_err(&mut lua, state.clone());
    runtime::print(&mut lua, state.clone());
    runtime::sleep(&mut lua, state.clone());
    runtime::status(&mut lua, state.clone());
    runtime::url_join(&mut lua, state.clone());
    runtime::url_parse(&mut lua, state.clone());

    debug!("Created lua context");

    (lua, state)
}

impl Script {
    pub fn load_unchecked<I: Into<String>>(code: I) -> Result<Script> {
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
            code: code.into(),
        })
    }

    pub fn run(&self, tx: Arc<Mutex<Box<Reporter>>>, arg: AnyLuaValue) -> Result<()> {
        let (mut lua, state) = ctx();

        debug!("Initializing lua module");
        lua.execute::<()>(&self.code)?;

        state.set_logger(tx);

        let run: Result<_> = lua.get("run")
            .ok_or_else(|| format_err!( "run undefined"));
        let mut run: hlua::LuaFunction<_> = run?;

        debug!("Starting lua script");
        let result: hlua::AnyLuaValue = run.call_with_args(arg)
            .map_err(|err| format_err!("execution failed: {:?}", err))?;

        debug!("Lua script terminated");

        if let Some(err) = state.error.lock().unwrap().take() {
            return Err(err);
        }

        use hlua::AnyLuaValue::*;
        match result {
            LuaString(x) => bail!("Script returned error: {:?}", x),
            _ => Ok(())
        }
    }

    #[cfg(test)]
    pub fn test(&self) -> Result<()> {
        use engine::tests::DummyReporter;
        self.run(DummyReporter::new(), AnyLuaValue::LuaNil)
    }
}
