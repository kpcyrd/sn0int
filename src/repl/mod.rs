use crate::config::Config;
use crate::errors::*;
use crate::engine::{ctx, Environment, DummyReporter};
use crate::engine::ctx::State;
use crate::geoip::{Maxmind, AsnDB, GeoIP};
use crate::hlua::AnyLuaValue;
use crate::psl::PslReader;
use crate::shell::readline::Readline;
use crate::runtime::format_lua;
use chrootable_https::Resolver;
use std::collections::HashMap;

mod complete;
use self::complete::ReplCompleter;


pub fn run(config: &Config) -> Result<()> {
    let keyring = Vec::new();
    let dns_config = Resolver::from_system()?;
    let proxy = config.network.proxy;
    let psl = PslReader::open()?;
    let geoip = GeoIP::open_reader()?;
    let asn = AsnDB::open_reader()?;

    let env = Environment {
        verbose: 0, // this doesn't do anything since we use a dummy reporter
        keyring,
        dns_config,
        proxy,
        options: HashMap::new(),
        blobs: Vec::new(),
        psl,
        geoip,
        asn,
    };

    let tx = DummyReporter::new();
    let (mut lua, state) = ctx::ctx(env, tx);

    println!(r#":: sn0int v{} lua repl
Assign variables with `a = sn0int_version()` and `return a` to print
Read the docs at https://sn0int.readthedocs.io/en/stable/reference.html
"#, env!("CARGO_PKG_VERSION"));

    let mut rl = Readline::with(ReplCompleter);
    loop {
        match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                match lua.execute::<AnyLuaValue>(&line) {
                    Ok(val) => {
                        if val != AnyLuaValue::LuaNil {
                            let mut out = String::new();
                            format_lua(&mut out, &val);
                            println!("{}", out);
                        }
                        if let Some(err) = state.last_error() {
                            println!("Error: {}", err);
                            state.clear_error();
                        }
                    },
                    Err(err) => {
                        println!("Fatal: {}", err);
                    }
                }
            },
            Err(_) => break,
        }
    }

    Ok(())
}
