use errors::*;

use engine::isolation::Worker;
use hlua;
use runtime;
use std::sync::{Arc, Mutex};
use worker::Event;


#[derive(Debug, Clone)]
pub struct State {
    error: Arc<Mutex<Option<Error>>>,
    logger: Arc<Mutex<Option<Arc<Mutex<Worker>>>>>,
}

impl State {
    pub fn new() -> State {
        State {
            error: Arc::new(Mutex::new(None)),
            logger: Arc::new(Mutex::new(None)),
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

    pub fn set_logger(&self, tx: Arc<Mutex<Worker>>) {
        let mut mtx = self.logger.lock().unwrap();
        *mtx = Some(tx);
    }

    pub fn info(&self, msg: String) {
        let mtx = self.logger.lock().unwrap();
        if let Some(mtx) = &*mtx {
            let mut tx = mtx.lock().unwrap();
            tx.send(&Event::Info(msg)).expect("Failed to write event");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    code: String,
}

fn ctx<'a>() -> (hlua::Lua<'a>, Arc<State>) {
    debug!("Creating lua context");
    let mut lua = hlua::Lua::new();
    lua.open_string();
    let state = Arc::new(State::new());

    // runtime::html_form(&mut lua, state.clone());
    // runtime::html_select(&mut lua, state.clone());
    // runtime::html_select_list(&mut lua, state.clone());
    // runtime::http_mksession(&mut lua, state.clone());
    // runtime::http_request(&mut lua, state.clone());
    // runtime::http_send(&mut lua, state.clone());
    runtime::info(&mut lua, state.clone());
    // runtime::json_decode(&mut lua, state.clone());
    // runtime::json_encode(&mut lua, state.clone());
    // runtime::last_err(&mut lua, state.clone());
    runtime::print(&mut lua, state.clone());
    runtime::sleep(&mut lua, state.clone());
    runtime::url_join(&mut lua, state.clone());
    runtime::url_parse(&mut lua, state.clone());

    debug!("Created lua context");

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

    pub fn run(&self, tx: Arc<Mutex<Worker>>) -> Result<()> {
        let (mut lua, state) = ctx();

        debug!("Initializing lua module");
        lua.execute::<()>(&self.code)?;

        state.set_logger(tx);

        let run: Result<_> = lua.get("run")
            .ok_or_else(|| format_err!( "run undefined"));
        let mut run: hlua::LuaFunction<_> = run?;

        debug!("Starting lua script");
        let _result: hlua::AnyLuaValue = run.call()
            .map_err(|err| format_err!("execution failed: {:?}", err))?;

        debug!("Lua script terminated");

        if let Some(err) = state.error.lock().unwrap().take() {
            return Err(err);
        }

        /*
        use hlua::AnyLuaValue::*;
        match result {
            LuaBoolean(x) => Ok(x),
            LuaString(x) => Err(format!("error: {:?}", x).into()),
            x => Err(format!("lua returned wrong type: {:?}", x).into()),
        }
        */

        Ok(())
    }
}
