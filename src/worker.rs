use errors::*;

use channel;
use engine::{self, Module};
use models::Object;
use serde_json;
use shell::Readline;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use term::Spinner;


#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Info(String),
    Error(String),
    Status(String),
    Object(Object),
    Done,
}

pub fn spawn(rl: &mut Readline, module: Module, arg: serde_json::Value, pretty_arg: &str) {
    let (tx, rx) = channel::bounded(1);

    let name = format!("{} ({:?})", module.canonical(), pretty_arg);
    let mut spinner = Spinner::random(format!("Running {}", name));

    let t = thread::spawn(move || {
        if let Err(err) = engine::isolation::spawn_module(module, tx.clone(), arg) {
            tx.send((Event::Error(err.to_string()), None));
        }
    });

    let mut failed = None;
    let timeout = Duration::from_millis(100);
    loop {
        select! {
            recv(rx, msg) => match msg {
                Some((Event::Info(info), _)) => spinner.log(&info),
                Some((Event::Error(error), _)) => {
                    failed = Some(error);
                    break;
                },
                Some((Event::Status(status), _)) => spinner.status(status),
                Some((Event::Object(object), tx)) => {
                    let result = rl.db().insert_generic(&object);
                    debug!("{:?} => {:?}", object, result);
                    let result = match result {
                        Ok((true, id)) => {
                            spinner.log(&format!("{}", object));
                            Ok(id)
                        },
                        Ok((_, id)) => Ok(id),
                        Err(err) => {
                            let err = err.to_string();
                            spinner.error(&err);
                            Err(err)
                        },
                    };

                    tx.expect("Failed to get db result channel")
                        .send(result).expect("Failed to send db result to channel");
                },
                Some((Event::Done, _)) => break,
                None => break, // channel closed
            },
            recv(channel::after(timeout)) => (),
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
                recv(rx, msg) => match msg {
                    Some(Event::Info(info)) => spinner.log(&info),
                    Some(Event::Error(error)) => spinner.error(&error),
                    Some(Event::Status(status)) => spinner.status(status),
                    Some(Event::Object(_)) => (),
                    Some(Event::Done) => break,
                    None => break, // channel closed
                },
                recv(channel::after(timeout)) => (),
            }
            spinner.tick();
        }
    });

    // run work in main thread
    let result = f()?;
    tx.send(Event::Done);

    t.join().expect("thread failed");

    let spinner = spinner.lock().unwrap();

    if clear {
        spinner.clear();
    } else {
        spinner.done();
    }

    Ok(result)
}
