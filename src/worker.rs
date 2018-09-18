use errors::*;

use engine::{self, Module};
use models::Object;
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

pub fn spawn(rl: &mut Readline, module: Module) {
    let (tx, rx) = mpsc::channel();

    let name = module.canonical();
    let mut spinner = Spinner::random(format!("Running {}", name));

    let t = thread::spawn(move || {
        tx.send(Event::Dummy).unwrap();

        if let Err(err) = engine::isolation::spawn_module(module, tx.clone()) {
            tx.send(Event::Error(err.to_string())).unwrap();
        }
    });

    let mut failed = None;
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Event::Dummy) => (),
            Ok(Event::Info(info)) => spinner.log(&info),
            Ok(Event::Error(error)) => {
                failed = Some(error);
                break;
            },
            Ok(Event::Status(status)) => spinner.status(status),
            Ok(Event::Object(object)) => match rl.db().insert_generic(&object) {
                Ok(true) => spinner.log(&format!("{:?}", object)),
                Ok(false) => (),
                Err(err) => spinner.error(&err.to_string()),
            },
            Ok(Event::Done) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        spinner.tick();
    }

    t.join().expect("thread failed");

    if let Some(fail) = failed {
        spinner.error(&fail);
        spinner.clear();
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
