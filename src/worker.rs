use std::time::Duration;
use std::sync::mpsc;
use std::thread;
use term::Spinner;

#[derive(Debug)]
pub enum Event {
    Info(String),
    Done,
}

pub fn spawn(task: &str) {
    let (tx, rx) = mpsc::channel();

    let mut spinner = Spinner::random(task.to_string());

    let t = thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));
        tx.send(Event::Info("ohai".to_string())).unwrap();
        thread::sleep(Duration::from_secs(1));
        tx.send(Event::Done).unwrap();
    });

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Event::Info(info)) => spinner.log(&info),
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
