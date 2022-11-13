use crate::errors::*;
use serde::{Serialize, Deserialize};

use crate::blobs::Blob;
use crate::channel;
use crate::cmd::run_cmd::Params;
use crate::db::{DbChange, Family};
use crate::db::ttl::Ttl;
use crate::engine::Module;
use crate::ipc;
use crate::ipc::parent::IpcParent;
use crate::models::*;
use crate::notify::{self, Notification};
use crate::ratelimits::{Ratelimiter, RatelimitResponse};
use crate::shell::Shell;
use sn0int_std::ratelimits::RatelimitSender;
use std::collections::HashMap;
use std::fmt::Write;
use std::result;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::thread;
use std::io::{Stdin, Read, BufRead, BufReader};
use std::net::SocketAddr;
use crate::term::{Spinner, StackedSpinners, SpinLogger};
use threadpool::ThreadPool;


type DbSender = mpsc::Sender<result::Result<DatabaseResponse, String>>;
pub type VoidSender = mpsc::Sender<result::Result<(), String>>;

#[derive(Debug, Serialize, Deserialize)]
pub enum DatabaseResponse {
    Inserted(i32),
    Updated(i32),
    Found(i32),
    NoChange(i32),
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Log(LogEvent),
    Database(Box<DatabaseEvent>),
    Stdio(StdioEvent),
    Ratelimit(RatelimitEvent),
    Blob(Blob),
    Exit(ExitEvent),
}

#[derive(Debug)]
pub enum Event2 {
    Start,
    Log(LogEvent),
    Database(Box<(DatabaseEvent, DbSender)>),
    Ratelimit((RatelimitEvent, RatelimitSender)),
    Blob((Blob, VoidSender)),
    Exit(ExitEvent),
}

#[derive(Debug)]
pub struct MultiEvent {
    pub name: String,
    pub event: Event2,
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
    #[inline]
    pub fn new(name: String, tx: channel::Sender<MultiEvent>) -> EventSender {
        EventSender {
            name,
            tx,
        }
    }

    #[inline]
    pub fn log(&self, log: LogEvent) {
        self.send(Event2::Log(log));
    }

    #[inline]
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

impl From<Result<()>> for ExitEvent {
    fn from(result: Result<()>) -> ExitEvent {
        match result {
            Ok(_) => ExitEvent::Ok,
            Err(err) => {
                let err = err.iter_chain()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(": ");
                ExitEvent::Err(err)
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LogEvent {
    Info(String),
    Debug(String),
    Success(String),
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
            LogEvent::Success(success) => spinner.success(&success),
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
    Activity(NewActivity),
    Select((Family, String)),
    Update((Family, String, Update)),
}

impl EventWithCallback for DatabaseEvent {
    type Payload = DatabaseResponse;

    fn with_callback(self, tx: mpsc::Sender<result::Result<Self::Payload, String>>) -> Event2 {
        Event2::Database(Box::new((self, tx)))
    }
}

impl DatabaseEvent {
    fn notify<T: SpinLogger>(rl: &mut Shell, spinner: &mut T, ratelimit: &mut Ratelimiter, topic: &str, subject: String) {
        if let Err(err) = notify::trigger_notify_event(rl, spinner, ratelimit, topic, &Notification {
            subject,
            body: None,
        }) {
            spinner.error(&format!("Failed to send notifications: {}", err));
        }
    }

    fn on_insert<T: SpinLogger>(rl: &mut Shell, spinner: &mut T, ratelimit: &mut Ratelimiter, family: &str, value: &str) {
        // TODO: also include fields, see update
        let log = format!("Adding {} {:?}", family, value);
        spinner.log(&log);

        let subject = format!("Added {} {:?}", family, value);
        let topic = format!("db:{}:{}:insert", family, value);
        Self::notify(rl, spinner, ratelimit, &topic, subject);
    }

    fn on_update<T: SpinLogger>(rl: &mut Shell, spinner: &mut T, ratelimit: &mut Ratelimiter, family: &str, value: &str, update: &Update) {
        spinner.log(&format!("Updating {} {:?} ({})", family, value, update.to_term_str()));

        // TODO: in the future we could consider firing multiple events, one for each column
        // TODO: this would be super noisy if a lot of fields change though
        let subject = format!("Updated {} {:?} ({})", family, value, update.to_plain_str());
        let topic = format!("db:{}:{}:update", family, value);
        Self::notify(rl, spinner, ratelimit, &topic, subject);
    }

    fn on_activity<T: SpinLogger>(rl: &mut Shell, spinner: &mut T, ratelimit: &mut Ratelimiter, object: &NewActivity, verbose: u64) {
        Self::spinner_log_new_activity(spinner, object, verbose);

        // TODO: we don't want to copy the match arms everywhere
        let mut subject = format!("New activity: {:?}", object.topic);
        if let Some(uniq) = &object.uniq {
            write!(subject, " ({:?})", uniq).expect("out of memory");
        }
        let topic = format!("activity:{}", object.topic);
        Self::notify(rl, spinner, ratelimit, &topic, subject);
    }

    fn spinner_log_new_activity<T: SpinLogger>(spinner: &mut T, object: &NewActivity, verbose: u64) {
        let mut log = format!("{:?} ", object.topic);
        if let Some(uniq) = &object.uniq {
            write!(log, "({:?}) ", uniq).expect("out of memory");
        }
        write!(log, "@ {}", object.time).expect("out of memory");

        if let (Some(ref lat), Some(ref lon)) = (object.latitude, object.longitude) {
            write!(log, " ({}, {}", lat, lon).expect("out of memory");
            if let Some(radius) = &object.radius {
                write!(log, " | {}m", radius).expect("out of memory");
            }
            log.push(')');
        }

        if verbose > 0 {
            write!(log, ": {}", object.content).expect("out of memory");
        }

        spinner.log(&log);
    }

    fn insert<T: SpinLogger>(rl: &mut Shell, spinner: &mut T, ratelimit: &mut Ratelimiter, object: Insert, ttl: Option<i32>, tx: DbSender, verbose: u64) {
        let db = rl.db();
        if verbose >= 1 {
            spinner.debug(&format!("Inserting: {:?}", object));
        }

        let result = db.insert_generic(object.clone());
        debug!("{:?} => {:?}", object, result);

        let result = match result {
            Ok(Some((DbChange::Insert, id))) => {
                match object.value(rl.db()) {
                    Ok(value) => {
                        if let Some(ttl) = ttl {
                            if let Err(err) = Ttl::create(&object, id, value.to_string(), ttl, db) {
                                spinner.error(&format!("Failed to set ttl: {:?}", err));
                            }
                        }

                        Self::on_insert(rl, spinner, ratelimit, object.family(), &value);
                    }
                    Err(err) => {
                        spinner.error(&format!("Failed to query necessary fields for {:?}: {:?}", object, err));
                    }
                }

                Ok(DatabaseResponse::Inserted(id))
            },
            Ok(Some((DbChange::Update(update), id))) => {
                if let Some(ttl) = ttl {
                    if let Err(err) = Ttl::bump(&object, id, ttl, db) {
                        spinner.error(&format!("Failed to set ttl: {:?}", err));
                    }
                }

                match object.value(rl.db()) {
                    Ok(value) => Self::on_update(rl, spinner, ratelimit, object.family(), &value, &update),
                    Err(err) => {
                        // TODO: this should be unreachable
                        spinner.error(&format!("Failed to get label for {:?}: {:?}", object, err));
                    },
                }

                Ok(DatabaseResponse::Updated(id))
            },
            Ok(Some((DbChange::None, id))) => {
                if let Some(ttl) = ttl {
                    if let Err(err) = Ttl::bump(&object, id, ttl, db) {
                        spinner.error(&format!("Failed to set ttl: {:?}", err));
                    }
                }

                Ok(DatabaseResponse::NoChange(id))
            },
            Ok(None) => Ok(DatabaseResponse::None),
            Err(err) => {
                let err = err.to_string();
                spinner.error(&err);
                Err(err)
            },
        };

        tx.send(result).expect("Failed to send db result to channel");
    }


    pub fn activity<T: SpinLogger>(rl: &mut Shell, spinner: &mut T, ratelimit: &mut Ratelimiter, object: NewActivity, tx: DbSender, verbose: u64) {
        let db = rl.db();
        let result = db.insert_activity(object.clone());
        debug!("{:?} => {:?}", object, result);

        let result = match result {
            Ok(true) => {
                Self::on_activity(rl, spinner, ratelimit, &object, verbose);
                Ok(DatabaseResponse::Inserted(0))
            },
            Ok(false) => Ok(DatabaseResponse::NoChange(0)),
            Err(err) => {
                let err = err.to_string();
                spinner.error(&err);
                Err(err)
            },
        };

        tx.send(result).expect("Failed to send db result to channel");
    }

    pub fn update<T: SpinLogger>(rl: &mut Shell, spinner: &mut T, ratelimit: &mut Ratelimiter, family: &str, value: &str, update: &Update, tx: DbSender, verbose: u64) {
        let db = rl.db();
        if verbose >= 1 {
            spinner.debug(&format!("Updating: {:?}", update));
        }

        let result = db.update_generic(update);
        debug!("{:?}: {:?} => {:?}", value, update, result);

        let result = match result {
            Ok(id) => {
                Self::on_update(rl, spinner, ratelimit, family, value, update);
                Ok(DatabaseResponse::Updated(id))
            },
            Err(err) => {
                let err = err.to_string();
                spinner.error(&err);
                Err(err)
            },
        };

        tx.send(result).expect("Failed to send db result to channel");
    }

    pub fn apply<T: SpinLogger>(self, rl: &mut Shell, spinner: &mut T, ratelimit: &mut Ratelimiter, tx: DbSender, verbose: u64) {
        match self {
            DatabaseEvent::Insert(object) => Self::insert(rl, spinner, ratelimit, object, None, tx, verbose),
            DatabaseEvent::InsertTtl((object, ttl)) => Self::insert(rl, spinner, ratelimit, object, Some(ttl), tx, verbose),
            DatabaseEvent::Activity(object) => Self::activity(rl, spinner, ratelimit, object, tx, verbose),
            DatabaseEvent::Select((family, value)) => {
                let db = rl.db();
                let result = match db.get_opt(&family, &value) {
                    Ok(Some(id)) => Ok(DatabaseResponse::Found(id)),
                    Ok(None) => Ok(DatabaseResponse::None),
                    Err(e) => Err(e.to_string()),
                };

                tx.send(result).expect("Failed to send db result to channel");
            },
            DatabaseEvent::Update((family, value, update)) => Self::update(rl, spinner, ratelimit, family.as_str(), &value, &update, tx, verbose),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StdioEvent {
    Readline,
    ToEnd,
}

impl StdioEvent {
    fn read_line(reader: &mut Option<BufReader<Stdin>>) -> Result<Option<String>> {
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

    fn read_to_end(reader: &mut Option<BufReader<Stdin>>) -> Result<Option<String>> {
        if let Some(ref mut reader) = reader {
            let mut buf = String::new();
            let len = reader.read_to_string(&mut buf)?;

            if len > 0 {
                Ok(Some(buf))
            } else {
                Ok(None)
            }
        } else {
            bail!("stdin is unavailable");
        }
    }

    pub fn apply(self, ipc_parent: &mut IpcParent, tx: &EventSender, reader: &mut Option<BufReader<Stdin>>) {
        let reply = match self {
            StdioEvent::Readline => Self::read_line(reader),
            StdioEvent::ToEnd => Self::read_to_end(reader),
        };
        let reply = reply.map_err(|e| e.to_string());
        ipc_parent.send_struct(reply, tx);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RatelimitEvent {
    key: String,
    passes: u32,
    time: u32,
}

impl EventWithCallback for RatelimitEvent {
    type Payload = RatelimitResponse;

    fn with_callback(self, tx: mpsc::Sender<result::Result<Self::Payload, String>>) -> Event2 {
        Event2::Ratelimit((self, tx))
    }
}

impl RatelimitEvent {
    #[inline]
    pub fn new(key: String, passes: u32, time: u32) -> RatelimitEvent {
        RatelimitEvent {
            key,
            passes,
            time,
        }
    }
}

pub fn spawn(rl: &mut Shell,
             module: &Module,
             ratelimit: &mut Ratelimiter,
             args: Vec<(serde_json::Value, Option<String>, Vec<Blob>)>,
             params: &Params,
             proxy: Option<SocketAddr>,
             user_agent: Option<String>,
             options: HashMap<String, String>,
) -> usize {
    // This function hangs if args is empty, so return early if that's the case
    if args.is_empty() {
        return 0;
    }

    let verbose = params.verbose;
    let has_stdin = params.stdin;
    let keyring = rl.keyring().request_keys(module);

    let mut stack = StackedSpinners::new();

    let (tx, rx) = channel::bounded(1);
    let pool = ThreadPool::new(params.threads);

    let mut expected = 0;
    debug!("Preparing to spawn scripts for {:?} structs", args.len());
    for (arg, pretty_arg, blobs) in args {
        let name = match pretty_arg {
            Some(pretty_arg) => format!("{:?}", pretty_arg),
            None => module.canonical(),
        };

        let tx = tx.clone();
        let module = module.clone();
        let keyring = keyring.clone();
        let user_agent = user_agent.clone();
        let options = options.clone();
        let signal_register = rl.signal_register().clone();
        pool.execute(move || {
            debug!("Thread pool job became active");
            let tx = EventSender::new(name, tx);

            if signal_register.ctrlc_received() {
                debug!("Thread pool job exits due to ctrl-c");
                tx.send(Event2::Exit(ExitEvent::Ok));
                return;
            }

            tx.send(Event2::Start);
            let event = match ipc::parent::run(module, &tx, arg, keyring, verbose, has_stdin, proxy, user_agent, options, blobs) {
                Ok(exit) => exit,
                // TODO: this should include the whole error chain
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
                        Event2::Database(tuple) => {
                            let (db, tx) = *tuple;
                            db.apply(rl, &mut stack.prefixed(name), ratelimit, tx, verbose)
                        },
                        Event2::Ratelimit((req, tx)) => ratelimit.pass(tx, &req.key, req.passes, req.time),
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
                    Some(Event::Ratelimit(_)) => (),
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

pub trait Task {
    fn initial_label(name: &str) -> String;

    fn name(&self) -> String;

    fn run(self, tx: &EventSender) -> Result<()>;
}

pub fn spawn_multi<T: Task, F>(tasks: Vec<T>, mut done_fn: F, threads: usize) -> Result<()>
    where
        F: FnMut(String),
        T: 'static + Send
{
    // This function hangs if args is empty, so return early if that's the case
    if tasks.is_empty() {
        return Ok(());
    }

    let mut expected = 0;
    let mut stack = StackedSpinners::new();

    let (tx, rx) = channel::bounded(1);
    let pool = ThreadPool::new(threads);

    for task in tasks {
        let tx = tx.clone();
        pool.execute(move || {
            let name = task.name();
            let tx = EventSender::new(name, tx);

            tx.send(Event2::Start);

            let exit = task.run(&tx);
            tx.send(Event2::Exit(exit.into()));
        });
        expected += 1;
    }

    let timeout = Duration::from_millis(100);
    loop {
        select! {
            recv(rx) -> msg => match msg.ok() {
                Some(event) => {
                    let (name, event) = (event.name, event.event);

                    match event {
                        Event2::Start => {
                            let label = T::initial_label(&name);
                            stack.add(name, label);
                        },
                        Event2::Log(log) => log.apply(&mut stack.prefixed(&name)),
                        Event2::Database(_) => (),
                        Event2::Ratelimit(_) => (),
                        Event2::Blob(_) => (),
                        Event2::Exit(event) => {
                            debug!("Received exit: {:?} -> {:?}", name, event);
                            stack.remove(&name);

                            match event {
                                ExitEvent::Ok => done_fn(name),
                                ExitEvent::Err(err) => {
                                    LogEvent::Error(err).apply(&mut stack.prefixed(&name));
                                },
                                ExitEvent::SetupFailed(_) => (),
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

    stack.clear();

    Ok(())
}
