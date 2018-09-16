use errors::*;
use engine::{Module, Event};
use serde_json;

use std::env;
use std::io::prelude::*;
use std::io::{self, BufReader, BufRead, Stdin, Stdout};
use std::sync::{mpsc, Arc, Mutex};
use std::process::{Command, Child, Stdio, ChildStdin, ChildStdout};


#[derive(Debug, Serialize, Deserialize)]
pub struct StartCommand {
    module: Module,
}

impl StartCommand {
    pub fn new(module: Module) -> StartCommand {
        StartCommand {
            module,
        }
    }
}

pub struct Supervisor {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Supervisor {
    pub fn setup(module: &Module) -> Result<Supervisor> {
        let exe = env::current_exe()
            .context("Failed to find current executable")?;

        let mut child = Command::new(exe)
            .arg("sandbox")
            .arg(&module.canonical())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to spawn child process")?;

        let stdin = child.stdin.take().expect("Failed to take child stdin");
        let stdout = child.stdout.take().expect("Failed to take child stdout");
        let stdout = BufReader::new(stdout);

        Ok(Supervisor {
            child,
            stdin,
            stdout,
        })
    }

    pub fn send_start(&mut self, module: Module) -> Result<()> {
        let start = StartCommand::new(module);
        let mut start = serde_json::to_string(&start)?;
        start.push('\n');
        self.stdin.write(start.as_bytes())?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<Event> {
        let mut line = String::new();
        let len = self.stdout.read_line(&mut line)?;

        let event = serde_json::from_str(&line[..len])?;
        Ok(event)
    }

    pub fn wait(&mut self) -> Result<()> {
        let exit = self.child.wait()
            .context("Failed to wait for child")?;

        if exit.success() {
            Ok(())
        } else {
            bail!("Child signaled error")
        }
    }
}

#[derive(Debug)]
pub struct Worker {
    stdin: Stdin,
    stdout: Stdout,
}

impl Worker {
    pub fn setup() -> Worker {
        let stdin = io::stdin();
        let stdout = io::stdout();

        Worker {
            stdin,
            stdout,
        }
    }

    pub fn recv_start(&mut self) -> Result<StartCommand> {
        let mut line = String::new();
        let len = self.stdin.read_line(&mut line)?;

        let event = serde_json::from_str(&line[..len])?;
        Ok(event)
    }

    pub fn send(&mut self, event: &Event) -> Result<()> {
        let mut event = serde_json::to_string(&event)?;
        event.push('\n');
        self.stdout.write(event.as_bytes())?;
        Ok(())
    }
}

pub fn spawn_module(module: Module, tx: mpsc::Sender<Event>) -> Result<()> {
    let mut supervisor = Supervisor::setup(&module)?;
    supervisor.send_start(module)?;

    loop {
        match supervisor.recv()? {
            Event::Done => break,
            event => tx.send(event).unwrap(),
        }
    }

    supervisor.wait()?;

    Ok(())
}

pub fn run_worker() -> Result<()> {
    let mut worker = Worker::setup();
    let start = worker.recv_start()?;

    let mtx = Arc::new(Mutex::new(worker));
    let result = start.module.run(mtx.clone());
    let mut worker = Arc::try_unwrap(mtx).expect("Failed to consume Arc")
                        .into_inner().expect("Failed to consume Mutex");

    if let Err(err) = result {
        worker.send(&Event::Error(err.to_string()))?;
    }

    worker.send(&Event::Done)?;

    Ok(())
}
