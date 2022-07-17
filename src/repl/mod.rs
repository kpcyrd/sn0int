use crate::config::Config;
use crate::errors::*;
use crate::engine::{ctx, Environment};
use crate::ipc::child::DummyIpcChild;
use crate::engine::ctx::{State, LuaState};
use crate::geoip::{Maxmind, AsnDB, GeoIP};
use crate::hlua::{Lua, AnyLuaValue};
use crate::paths;
use crate::psl::PslReader;
use crate::shell::readline::Readline;
use crate::runtime::format_lua;
use chrootable_https::Resolver;
use std::collections::HashMap;
use std::sync::Arc;

mod complete;
use self::complete::ReplCompleter;
mod tokenize;

pub struct Repl<'a> {
    rl: Readline<ReplCompleter>,
    lua: Lua<'a>,
    state: Arc<LuaState>,
}

impl<'a> Repl<'a> {
    pub fn new(lua: Lua<'a>, state: Arc<LuaState>) -> Result<Repl<'a>> {
        let rl = Readline::with(ReplCompleter::default())?;
        Ok(Repl {
            rl,
            lua,
            state,
        })
    }

    fn update_globals(&mut self) {
        let mut globals = Vec::new();
        for (k, _) in self.lua.globals_table().iter::<String, AnyLuaValue>().flatten() {
            globals.push(k);
        }
        if let Some(helper) = self.rl.helper_mut() {
            debug!("updating globals: {:?}", globals);
            helper.set(globals);
        }
    }

    pub fn run(&mut self) {
        loop {
            self.update_globals();

            match self.rl.readline("> ") {
                Ok(line) => {
                    self.rl.add_history_entry(line.as_str());
                    self.exec(&line);
                },
                Err(_) => break,
            }
        }
    }

    pub fn exec(&mut self, line: &str) {
        match self.lua.execute::<AnyLuaValue>(line) {
            Ok(val) => {
                if val != AnyLuaValue::LuaNil {
                    let mut out = String::new();
                    format_lua(&mut out, &val).expect("out of memory");
                    println!("{}", out);
                }
                if let Some(err) = self.state.last_error() {
                    println!("Error: {}", err);
                    self.state.clear_error();
                }
            },
            Err(err) => {
                println!("Fatal: {}", err);
            }
        }
    }
}

pub fn run(config: &Config) -> Result<()> {
    let keyring = Vec::new();
    let dns_config = Resolver::from_system_v4()?;
    let proxy = config.network.proxy;
    let user_agent = config.network.user_agent.clone();

    let cache_dir = paths::cache_dir()?;
    let psl = PslReader::open(&cache_dir)?;
    let geoip = GeoIP::try_open_reader(&cache_dir)?;
    let asn = AsnDB::try_open_reader(&cache_dir)?;

    let env = Environment {
        verbose: 0, // this doesn't do anything since we use a dummy ipc child
        keyring,
        dns_config,
        proxy,
        user_agent,
        options: HashMap::new(),
        blobs: Vec::new(),
        psl,
        geoip,
        asn,
    };

    let tx = DummyIpcChild::create();
    let (lua, state) = ctx::ctx(env, tx);
    let mut repl = Repl::new(lua, state)?;

    println!(r#":: sn0int v{} lua repl
Assign variables with `a = sn0int_version()` and `return a` to print
Read the docs at https://sn0int.readthedocs.io/en/stable/reference.html
"#, env!("CARGO_PKG_VERSION"));

    repl.run();

    Ok(())
}
