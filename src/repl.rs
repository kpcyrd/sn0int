use crate::errors::*;
use crate::engine::{ctx, Environment, DummyReporter};
use crate::engine::ctx::State;
use crate::geoip::{Maxmind, AsnDB, GeoIP};
use crate::hlua::AnyLuaValue;
use crate::psl::PslReader;
use crate::runtime::format_lua;
use chrootable_https::Resolver;
use std::collections::HashMap;
use rustyline::{CompletionType, EditMode, Editor, KeyPress, Movement, Word, At};


pub fn run() -> Result<()> {
    let keyring = Vec::new();
    let dns_config = Resolver::from_system()?;
    let proxy = None;
    let psl = PslReader::open()?;
    let geoip = GeoIP::open_reader()?;
    let asn = AsnDB::open_reader()?;

    let env = Environment {
        verbose: 0,
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

    let rl_config = rustyline::Config::builder()
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let mut rl: Editor<()> = Editor::with_config(rl_config);
    rl.bind_sequence(KeyPress::ControlLeft, rustyline::Cmd::Move(Movement::BackwardWord(1, Word::Big)));
    rl.bind_sequence(KeyPress::ControlRight, rustyline::Cmd::Move(Movement::ForwardWord(1, At::Start, Word::Big)));

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
