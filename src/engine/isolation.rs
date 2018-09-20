use errors::*;
use engine::{Module, Event, Reporter};
use serde_json;

use std::env;
use std::io::prelude::*;
use std::io::{self, BufReader, BufRead, Stdin, Stdout};
use std::sync::{mpsc, Arc, Mutex};
use std::process::{Command, Child, Stdio, ChildStdin, ChildStdout};


#[derive(Debug, Serialize, Deserialize)]
pub struct StartCommand {
    module: Module,
    arg: serde_json::Value,
}

impl StartCommand {
    pub fn new(module: Module, arg: serde_json::Value) -> StartCommand {
        StartCommand {
            module,
            arg
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

    pub fn send_start(&mut self, module: Module, arg: serde_json::Value) -> Result<()> {
        let start = StartCommand::new(module, arg);
        let mut start = serde_json::to_string(&start)?;
        start.push('\n');
        self.stdin.write_all(start.as_bytes())?;
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
pub struct StdioReporter {
    stdin: Stdin,
    stdout: Stdout,
}

impl StdioReporter {
    pub fn setup() -> StdioReporter {
        let stdin = io::stdin();
        let stdout = io::stdout();

        StdioReporter {
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
}

impl Reporter for StdioReporter {
    fn send(&mut self, event: &Event) -> Result<()> {
        let mut event = serde_json::to_string(&event)?;
        event.push('\n');
        self.stdout.write_all(event.as_bytes())?;
        Ok(())
    }
}

pub fn spawn_module(module: Module, tx: mpsc::Sender<Event>, arg: serde_json::Value) -> Result<()> {
    let mut supervisor = Supervisor::setup(&module)?;
    supervisor.send_start(module, arg)?;

    loop {
        match supervisor.recv()? {
            Event::Done => break,
            Event::Error(err) => {
                tx.send(Event::Error(err)).unwrap();
                break;
            },
            event => tx.send(event).unwrap(),
        }
    }

    supervisor.wait()?;

    Ok(())
}

pub fn run_worker() -> Result<()> {
    let mut reporter = StdioReporter::setup();
    let start = reporter.recv_start()?;

    let mtx: Arc<Mutex<Box<Reporter>>> = Arc::new(Mutex::new(Box::new(reporter)));
    let result = start.module.run(mtx.clone(), start.arg.into());
    let mut reporter = Arc::try_unwrap(mtx).expect("Failed to consume Arc")
                        .into_inner().expect("Failed to consume Mutex");

    if let Err(err) = result {
        reporter.send(&Event::Error(err.to_string()))?;
    } else {
        reporter.send(&Event::Done)?;
    }

    Ok(())
}
