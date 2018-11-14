use errors::*;

use channel;
use db::{Database, DbChange, Family};
use engine::{self, Module};
use models::*;
use serde_json;
use shell::Readline;
use std::result;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::thread;
use term::{Spinner, StackedSpinners, SpinLogger};
use threadpool::ThreadPool;


type DbSender = mpsc::Sender<result::Result<Option<i32>, String>>;

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Log(LogEvent),
    Database(DatabaseEvent),
    Exit(ExitEvent),
}

#[derive(Debug)]
pub enum Event2 {
    Start,
    Log(LogEvent),
    Database((DatabaseEvent, DbSender)),
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

#[derive(Debug, Serialize, Deserialize)]
pub enum ExitEvent {
    Ok,
    Err(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LogEvent {
    Info(String),
    Error(String),
    Status(String),
}

impl LogEvent {
    pub fn apply<T: SpinLogger>(self, spinner: &mut T) {
        match self {
            LogEvent::Info(info) => spinner.log(&info),
            LogEvent::Error(error) => spinner.error(&error),
            LogEvent::Status(status) => spinner.status(status),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DatabaseEvent {
    Insert(Insert),
    Select((Family, String)),
    Update((String, Update)),
}

impl DatabaseEvent {
    pub fn apply<T: SpinLogger>(self, tx: DbSender, spinner: &mut T, db: &Database) {
        match self {
            DatabaseEvent::Insert(object) => {
                let result = db.insert_generic(&object);
                debug!("{:?} => {:?}", object, result);
                let result = match result {
                    Ok((DbChange::Insert, id)) => {
                        // TODO: replace id with actual object(?)
                        if let Ok(obj) = object.printable(db) {
                            spinner.log(&obj.to_string());
                        } else {
                            spinner.error(&format!("Failed to query necessary fields for {:?}", object));
                        }
                        Ok(Some(id))
                    },
                    Ok((DbChange::Update(update), id)) => {
                        // TODO: replace id with actual object(?)
                        spinner.log(&format!("Updating {:?} ({})", object.value(), update));
                        Ok(Some(id))
                    },
                    Ok((DbChange::None, id)) => Ok(Some(id)),
                    Err(err) => {
                        let err = err.to_string();
                        spinner.error(&err);
                        Err(err)
                    },
                };

                tx.send(result).expect("Failed to send db result to channel");
            },
            DatabaseEvent::Select((family, value)) => {
                let result = db.get_opt(&family, &value)
                    .map_err(|e| e.to_string());

                tx.send(result).expect("Failed to send db result to channel");
            },
            DatabaseEvent::Update((object, update)) => {
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

pub fn spawn(rl: &mut Readline, module: &Module, args: Vec<(serde_json::Value, Option<String>)>) {
    // This function hangs if args is empty, so return early if that's the case
    if args.is_empty() {
        return;
    }

    let mut stack = StackedSpinners::new();

    let (tx, rx) = channel::bounded(1);
    let pool = ThreadPool::new(4); // TODO: this should by dynamic

    let mut expected = 0;
    for (arg, pretty_arg) in args {
        let name = match pretty_arg {
            Some(pretty_arg) => format!("{} ({:?})", module.canonical(), pretty_arg),
            None => module.canonical(),
        };

        let tx = tx.clone();
        let module = module.clone();
        let signal_register = rl.signal_register().clone();
        pool.execute(move || {
            let tx = EventSender::new(name, tx);

            if signal_register.ctrlc_received() {
                tx.send(Event2::Exit(ExitEvent::Ok));
                return;
            }

            tx.send(Event2::Start);
            let event = match engine::isolation::spawn_module(module, &tx, arg) {
                Ok(_) => ExitEvent::Ok,
                Err(err) => ExitEvent::Err(err.to_string()),
            };
            tx.send(Event2::Exit(event));
        });
        expected += 1;
    }

    let mut failed = Vec::new();
    let timeout = Duration::from_millis(100);
    loop {
        select! {
            recv(rx) -> msg => match msg.ok() {
                Some(event) => {
                    let (name, event) = (event.name, event.event);

                    match event {
                        Event2::Start => {
                            let label = format!("Runnig {}", name);
                            stack.add(name, label);
                        },
                        Event2::Log(log) => log.apply(&mut stack),
                        Event2::Database((db, tx)) => db.apply(tx, &mut stack, rl.db()),
                        Event2::Exit(event) => {
                            stack.remove(&name);
                            if let ExitEvent::Err(error) = event {
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

    for (name, fail) in failed {
        stack.error(&format!("Failed {}: {}", name, fail));
    }

    stack.clear();
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
                    // TODO: refactor
                    Some(Event::Exit(ExitEvent::Ok)) => break,
                    Some(Event::Exit(ExitEvent::Err(error))) => spinner.error(&error),
                    None => break, // channel closed
                },
                default(timeout) => (),
            }
            spinner.tick();
        }
    });

    // run work in main thread
    let result = f()?;
    tx.send(Event::Exit(ExitEvent::Ok))?;

    t.join().expect("thread failed");

    let spinner = spinner.lock().unwrap();

    if clear {
        spinner.clear();
    } else {
        spinner.done();
    }

    Ok(result)
}
