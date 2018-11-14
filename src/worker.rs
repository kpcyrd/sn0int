use errors::*;

use channel;
use db::{DbChange, Family};
use engine::{self, Module};
use models::*;
use serde_json;
use shell::Readline;
use std::result;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::thread;
use term::{Spinner};


#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Log(LogEvent),
    Database(DatabaseEvent),
    Exit(ExitEvent),
}

#[derive(Debug)]
pub enum Event2 {
    Log(LogEvent),
    Database((DatabaseEvent, mpsc::Sender<result::Result<Option<i32>, String>>)),
    Exit(ExitEvent),
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

#[derive(Debug, Serialize, Deserialize)]
pub enum DatabaseEvent {
    Insert(Insert),
    Select((Family, String)),
    Update((String, Update)),
}

pub fn spawn(rl: &mut Readline, module: Module, arg: serde_json::Value, pretty_arg: &Option<String>) {
    let (tx, rx) = channel::bounded(1);

    let name = match pretty_arg {
        Some(pretty_arg) => format!("{} ({:?})", module.canonical(), pretty_arg),
        None => module.canonical(),
    };
    let mut spinner = Spinner::random(format!("Running {}", name));

    let t = thread::spawn(move || {
        if let Err(err) = engine::isolation::spawn_module(module, tx.clone(), arg) {
            tx.send(Event2::Log(LogEvent::Error(err.to_string()))).unwrap();
        }
    });

    let mut failed = None;
    let timeout = Duration::from_millis(100);
    loop {
        select! {
            recv(rx) -> msg => match msg.ok() {
                Some(Event2::Log(LogEvent::Info(info))) => spinner.log(&info),
                Some(Event2::Log(LogEvent::Error(error))) => spinner.error(&error),
                Some(Event2::Log(LogEvent::Status(status))) => spinner.status(status),
                Some(Event2::Database((DatabaseEvent::Insert(object), tx))) => {
                    let result = rl.db().insert_generic(&object);
                    debug!("{:?} => {:?}", object, result);
                    let result = match result {
                        Ok((DbChange::Insert, id)) => {
                            // TODO: replace id with actual object(?)
                            if let Ok(obj) = object.printable(rl.db()) {
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
                Some(Event2::Database((DatabaseEvent::Select((family, value)), tx))) => {
                    let result = rl.db().get_opt(&family, &value)
                        .map_err(|e| e.to_string());

                    tx.send(result).expect("Failed to send db result to channel");
                },
                Some(Event2::Database((DatabaseEvent::Update((object, update)), tx))) => {
                    let result = rl.db().update_generic(&update);
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
                Some(Event2::Exit(event)) => {
                    if let ExitEvent::Err(error) = event {
                        failed = Some(error);
                    }
                    break;
                },
                None => break, // channel closed
            },
            default(timeout) => (),
        }
        spinner.tick();
    }

    t.join().expect("thread failed");

    if let Some(fail) = failed {
        spinner.fail(&format!("Failed {}: {}", name, fail));
    } else {
        spinner.clear();
    }
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
                    Some(Event::Log(LogEvent::Info(info))) => spinner.log(&info),
                    Some(Event::Log(LogEvent::Error(error))) => spinner.error(&error),
                    Some(Event::Log(LogEvent::Status(status))) => spinner.status(status),
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
