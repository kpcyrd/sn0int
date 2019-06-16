use crate::errors::*;

use crate::blobs::Blob;
use crate::channel;
use crate::cmd::run_cmd::Params;
use crate::db::{Database, DbChange, Family};
use crate::db::ttl::Ttl;
use crate::engine::{self, Module};
use crate::engine::isolation::Supervisor;
use crate::models::*;
use serde_json;
use crate::shell::Readline;
use std::collections::HashMap;
use std::result;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::thread;
use std::io::{Stdin, BufRead, BufReader};
use std::net::SocketAddr;
use crate::term::{Spinner, StackedSpinners, SpinLogger};
use threadpool::ThreadPool;


type DbSender = mpsc::Sender<result::Result<Option<i32>, String>>;
pub type VoidSender = mpsc::Sender<result::Result<(), String>>;

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Log(LogEvent),
    Database(DatabaseEvent),
    Stdio(StdioEvent),
    Blob(Blob),
    Exit(ExitEvent),
}

#[derive(Debug)]
pub enum Event2 {
    Start,
    Log(LogEvent),
    Database((DatabaseEvent, DbSender)),
    Blob((Blob, VoidSender)),
    Exit(ExitEvent),
}

#[derive(Debug)]
pub struct MultiEvent {
    name: String,
    event: Event2,
}

impl MultiEvent {
    pub fn new<I: Into<String>>(name: I, event: Event2) -> MultiEvent {
        MultiEvent {
            name: name.into(),
            event,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventSender {
    name: String,
    tx: channel::Sender<MultiEvent>,
}

impl EventSender {
    pub fn new(name: String, tx: channel::Sender<MultiEvent>) -> EventSender {
        EventSender {
            name,
            tx,
        }
    }

    pub fn send(&self, event: Event2) {
        self.tx.send(MultiEvent::new(self.name.clone(), event)).unwrap();
    }
}

pub trait EventWithCallback {
    type Payload;

    fn with_callback(self, tx: mpsc::Sender<result::Result<Self::Payload, String>>) -> Event2;
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ExitEvent {
    Ok,
    Err(String),
    SetupFailed(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LogEvent {
    Info(String),
    Debug(String),
    Error(String),
    Warn(String),
    WarnOnce(String),
    Status(String),
}

impl LogEvent {
    pub fn apply<T: SpinLogger>(self, spinner: &mut T) {
        match self {
            LogEvent::Info(info) => spinner.log(&info),
            LogEvent::Debug(debug) => spinner.debug(&debug),
            LogEvent::Error(error) => spinner.error(&error),
            LogEvent::Warn(warn) => spinner.warn(&warn),
            LogEvent::WarnOnce(warn) => spinner.warn_once(&warn),
            LogEvent::Status(status) => spinner.status(status),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DatabaseEvent {
    Insert(Insert),
    InsertTtl((Insert, i32)),
    Select((Family, String)),
    Update((String, Update)),
}

impl EventWithCallback for DatabaseEvent {
    type Payload = Option<i32>;

    fn with_callback(self, tx: mpsc::Sender<result::Result<Self::Payload, String>>) -> Event2 {
        Event2::Database((self, tx))
    }
}

impl DatabaseEvent {
    pub fn insert<T: SpinLogger>(object: Insert, ttl: Option<i32>, tx: DbSender, spinner: &mut T, db: &Database, verbose: u64) {
        if verbose >= 1 {
            spinner.debug(&format!("Inserting: {:?}", object));
        }

        let result = db.insert_generic(object.clone());
        debug!("{:?} => {:?}", object, result);

        let result = match result {
            Ok(Some((DbChange::Insert, id))) => {
                if let Some(ttl) = ttl {
                    if let Err(err) = Ttl::create(&object, id, ttl, db) {
                        spinner.error(&format!("Failed to set ttl: {:?}", err));
                    }
                }

                // TODO: replace id with actual object(?)
                if let Ok(obj) = object.printable(db) {
                    spinner.log(&obj.to_string());
                } else {
                    spinner.error(&format!("Failed to query necessary fields for {:?}", object));
                }
                Ok(Some(id))
            },
            Ok(Some((DbChange::Update(update), id))) => {
                if let Some(ttl) = ttl {
                    if let Err(err) = Ttl::bump(&object, id, ttl, db) {
                        spinner.error(&format!("Failed to set ttl: {:?}", err));
                    }
                }

                // TODO: replace id with actual object(?)
                match object.label(&db) {
                    Ok(label) => {
                        spinner.log(&format!("Updating {} ({})", label, update));
                    },
                    Err(err) => {
                        // TODO: this should be unreachable
                        spinner.error(&format!("Failed to get label for {:?}: {:?}", object, err));
                    },
                }
                Ok(Some(id))
            },
            Ok(Some((DbChange::None, id))) => {
                if let Some(ttl) = ttl {
                    if let Err(err) = Ttl::bump(&object, id, ttl, db) {
                        spinner.error(&format!("Failed to set ttl: {:?}", err));
                    }
                }

                Ok(Some(id))
            },
            Ok(None) => Ok(None),
            Err(err) => {
                let err = err.to_string();
                spinner.error(&err);
                Err(err)
            },
        };

        tx.send(result).expect("Failed to send db result to channel");
    }

    pub fn apply<T: SpinLogger>(self, tx: DbSender, spinner: &mut T, db: &Database, verbose: u64) {
        match self {
            DatabaseEvent::Insert(object) => Self::insert(object, None, tx, spinner, db, verbose),
            DatabaseEvent::InsertTtl((object, ttl)) => Self::insert(object, Some(ttl), tx, spinner, db, verbose),
            DatabaseEvent::Select((family, value)) => {
                let result = db.get_opt(&family, &value)
                    .map_err(|e| e.to_string());

                tx.send(result).expect("Failed to send db result to channel");
            },
            DatabaseEvent::Update((object, update)) => {
                if verbose >= 1 {
                    spinner.debug(&format!("Updating: {:?}", update));
                }

                let result = db.update_generic(&update);
                debug!("{:?}: {:?} => {:?}", object, update, result);
                let result = result
                    .map(Some)
                    .map_err(|e| e.to_string());

                if let Err(ref err) = result {
                    spinner.error(&err);
                } else {
                    // TODO: bring this somewhat closer to upsert code
                    spinner.log(&format!("Updating {:?} ({})", object, update));
                }

                tx.send(result).expect("Failed to send db result to channel");
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StdioEvent {
}

impl StdioEvent {
    fn read(reader: &mut Option<BufReader<Stdin>>) -> Result<Option<String>> {
        if let Some(ref mut reader) = reader {
            let mut line = String::new();
            let len = reader.read_line(&mut line)?;
            debug!("stdin: {:?}", line);

            if len > 0 {
                Ok(Some(line))
            } else {
                Ok(None)
            }
        } else {
            bail!("stdin is unavailable");
        }
    }

    pub fn apply(self, supervisor: &mut Supervisor, tx: &EventSender, reader: &mut Option<BufReader<Stdin>>) {
        let reply = Self::read(reader)
            .map_err(|e| e.to_string());
        supervisor.send_struct(reply, tx);
    }
}

pub fn spawn(rl: &mut Readline, module: &Module, args: Vec<(serde_json::Value, Option<String>, Vec<Blob>)>, params: &Params, proxy: Option<SocketAddr>, options: HashMap<String, String>) -> usize {
    // This function hangs if args is empty, so return early if that's the case
    if args.is_empty() {
        return 0;
    }

    let verbose = params.verbose;
    let has_stdin = params.stdin;
    let keyring = rl.keyring().request_keys(&module);

    let mut stack = StackedSpinners::new();

    let (tx, rx) = channel::bounded(1);
    let pool = ThreadPool::new(params.threads);

    let mut expected = 0;
    for (arg, pretty_arg, blobs) in args {
        let name = match pretty_arg {
            Some(pretty_arg) => format!("{:?}", pretty_arg),
            None => module.canonical(),
        };

        let tx = tx.clone();
        let module = module.clone();
        let keyring = keyring.clone();
        let options = options.clone();
        let signal_register = rl.signal_register().clone();
        pool.execute(move || {
            let tx = EventSender::new(name, tx);

            if signal_register.ctrlc_received() {
                tx.send(Event2::Exit(ExitEvent::Ok));
                return;
            }

            tx.send(Event2::Start);
            let event = match engine::isolation::spawn_module(module, &tx, arg, keyring, verbose, has_stdin, proxy, options, blobs) {
                Ok(exit) => exit,
                Err(err) => ExitEvent::SetupFailed(err.to_string()),
            };
            tx.send(Event2::Exit(event));
        });
        expected += 1;
    }

    let mut errors = 0;
    let mut failed = Vec::new();
    let timeout = Duration::from_millis(100);
    loop {
        select! {
            recv(rx) -> msg => match msg.ok() {
                Some(event) => {
                    let (name, event) = (event.name, event.event);

                    match event {
                        Event2::Start => {
                            let label = format!("Investigating {}", name);
                            stack.add(name, label);
                        },
                        Event2::Log(log) => log.apply(&mut stack.prefixed(name)),
                        Event2::Database((db, tx)) => db.apply(tx, &mut stack.prefixed(name), rl.db(), verbose),
                        Event2::Blob((blob, tx)) => rl.store_blob(tx, &blob),
                        Event2::Exit(event) => {
                            debug!("Received exit: {:?} -> {:?}", name, event);
                            stack.remove(&name);

                            if ExitEvent::Ok != event {
                                trace!("bumping error counter");
                                errors += 1;
                            }

                            if let ExitEvent::SetupFailed(error) = event {
                                failed.push((name, error));
                            }

                            // if every task reported back, exit
                            expected -= 1;
                            info!("spawn_all is expecting {} more results", expected);
                            if expected == 0 {
                                break;
                            }
                        },
                    }
                },
                None => break, // channel closed
            },
            default(timeout) => (),
        }
        stack.tick();
    }

    for (name, fail) in &failed {
        stack.error(&format!("Failed {}: {}", name, fail));
    }

    stack.clear();

    errors
}

pub fn spawn_fn<F, T>(label: &str, f: F, clear: bool) -> Result<T>
        where F: FnOnce() -> Result<T> {
    let (tx, rx) = channel::bounded(1);

    let spinner = Arc::new(Mutex::new(Spinner::random(label.to_string())));
    let spinner2 = spinner.clone();
    let t = thread::spawn(move || {
        let mut spinner = spinner2.lock().unwrap();

        let timeout = Duration::from_millis(100);
        loop {
            select! {
                recv(rx) -> msg => match msg.ok() {
                    Some(Event::Log(log)) => log.apply(&mut *spinner),
                    Some(Event::Database(_)) => (),
                    Some(Event::Stdio(_)) => (),
                    Some(Event::Blob(_)) => (),
                    // TODO: refactor
                    Some(Event::Exit(ExitEvent::Ok)) => break,
                    Some(Event::Exit(ExitEvent::Err(error))) => spinner.error(&error),
                    Some(Event::Exit(ExitEvent::SetupFailed(error))) => spinner.error(&error),
                    None => break, // channel closed
                },
                default(timeout) => (),
            }
            spinner.tick();
        }
    });

    // run work in main thread
    let result = f();
    tx.send(Event::Exit(ExitEvent::Ok))?;

    t.join().expect("thread failed");

    let spinner = spinner.lock().unwrap();

    if clear || result.is_err() {
        spinner.clear();
    } else {
        spinner.done();
    }

    result
}
