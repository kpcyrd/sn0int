use errors::*;

use engine::{self, Module};
use std::time::Duration;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use term::Spinner;


#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Info(String),
    Error(String),
    Done,
}

pub fn spawn(module: Module) {
    let (tx, rx) = mpsc::channel();

    let mut spinner = Spinner::random(format!("Running {}", module.canonical()));

    let t = thread::spawn(move || {
        if let Err(err) = engine::isolation::spawn_module(module, tx.clone()) {
            tx.send(Event::Error(err.to_string())).unwrap();
        }

        /*
        if let Err(err) = module.run(tx.clone()) {
            tx.send(Event::Error(err)).unwrap();
        }
        */

        // thread::sleep(Duration::from_secs(3));
        // tx.send(Event::Info("ohai".to_string())).unwrap();
        // thread::sleep(Duration::from_secs(1));
        tx.send(Event::Done).unwrap();
    });

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Event::Info(info)) => spinner.log(&info),
            Ok(Event::Error(error)) => spinner.error(&error.to_string()),
            Ok(Event::Done) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        spinner.tick();
    }

    t.join().expect("thread failed");

    // spinner.clear();
    spinner.done();
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
                Ok(Event::Info(info)) => spinner.log(&info),
                Ok(Event::Error(error)) => spinner.error(&error.to_string()),
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
