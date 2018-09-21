use errors::*;

use engine::{self, Module};
use models::Object;
use serde_json;
use shell::Readline;
use std::time::Duration;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use term::Spinner;


#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    // The dummy event type was introduced due to this:
    // https://github.com/rust-lang/rust/issues/54267
    // https://github.com/rust-lang/rust/issues/39364
    // After this is resolved the code that is sending dummy messages can be deleted
    Dummy,
    Info(String),
    Error(String),
    Status(String),
    Object(Object),
    Done,
}

pub fn spawn(rl: &mut Readline, module: Module, arg: serde_json::Value, pretty_arg: &str) {
    let (tx, rx) = mpsc::channel();

    let name = format!("{} ({:?})", module.canonical(), pretty_arg);
    let mut spinner = Spinner::random(format!("Running {}", name));

    let t = thread::spawn(move || {
        // Send two messages and hopefully fix occasional panics
        // https://github.com/rust-lang/rust/issues/39364#issuecomment-421853099
        tx.send((Event::Dummy, None)).unwrap();
        tx.send((Event::Dummy, None)).unwrap();

        if let Err(err) = engine::isolation::spawn_module(module, tx.clone(), arg) {
            tx.send((Event::Error(err.to_string()), None)).unwrap();
        }
    });

    let mut failed = None;
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok((Event::Dummy, _)) => (),
            Ok((Event::Info(info), _)) => spinner.log(&info),
            Ok((Event::Error(error), _)) => {
                failed = Some(error);
                break;
            },
            Ok((Event::Status(status), _)) => spinner.status(status),
            Ok((Event::Object(object), tx)) => {
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
            Ok((Event::Done, _)) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        spinner.tick();
    }

    t.join().expect("thread failed");

    if let Some(fail) = failed {
        spinner.fail(&format!("Failed {}: {}", name, fail));
    } else {
        spinner.finish(format!("Finished {}", name));
    }
}

pub fn spawn_fn<F, T>(label: &str, f: F, clear: bool) -> Result<T>
        where F: FnOnce() -> Result<T> {
    let (tx, rx) = mpsc::channel();

    let spinner = Arc::new(Mutex::new(Spinner::random(label.to_string())));
    let spinner2 = spinner.clone();
    let t = thread::spawn(move || {
        let mut spinner = spinner2.lock().unwrap();

        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Event::Dummy) => (),
                Ok(Event::Info(info)) => spinner.log(&info),
                Ok(Event::Error(error)) => spinner.error(&error),
                Ok(Event::Status(status)) => spinner.status(status),
                Ok(Event::Object(_)) => (),
                Ok(Event::Done) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            spinner.tick();
        }
    });

    // run work in main thread
    let result = f()?;
    tx.send(Event::Done).unwrap();

    t.join().expect("thread failed");

    let spinner = spinner.lock().unwrap();

    if clear {
        spinner.clear();
    } else {
        spinner.done();
    }

    Ok(result)
}
