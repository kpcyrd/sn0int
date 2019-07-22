use crate::errors::*;
use chrootable_https::dns::Resolver;
use crate::blobs::Blob;
use crate::engine::{Environment, Module, Reporter};
use crate::geoip::MaxmindReader;
use crate::keyring::KeyRingEntry;
use crate::psl::PslReader;
use serde_json;
use crate::worker::{Event, Event2, LogEvent, ExitEvent, EventSender, EventWithCallback};

use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::io::prelude::*;
use std::io::{self, BufReader, BufRead, stdin, Stdin, Stdout};
use std::net::SocketAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::process::{Command, Child, Stdio, ChildStdin, ChildStdout};


#[derive(Debug, Serialize, Deserialize)]
pub struct StartCommand {
    verbose: u64,
    keyring: Vec<KeyRingEntry>,
    dns_config: Resolver,
    proxy: Option<SocketAddr>,
    options: HashMap<String, String>,
    module: Module,
    arg: serde_json::Value,
    blobs: Vec<Blob>,
}

impl StartCommand {
    pub fn new(verbose: u64,
               keyring: Vec<KeyRingEntry>,
               dns_config: Resolver,
               proxy: Option<SocketAddr>,
               options: HashMap<String, String>,
               module: Module,
               arg: serde_json::Value,
               blobs: Vec<Blob>,
    ) -> StartCommand {
        StartCommand {
            verbose,
            keyring,
            dns_config,
            proxy,
            options,
            module,
            arg,
            blobs,
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
        let exe = match env::current_exe() {
            Ok(exe) => exe.into_os_string(),
            _ => OsString::from("sn0int"),
        };

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

    pub fn send_start(&mut self, start: &StartCommand) -> Result<()> {
        let start = serde_json::to_value(&start)?;
        self.send(&start)?;
        Ok(())
    }

    pub fn send(&mut self, value: &serde_json::Value) -> Result<()> {
        let mut value = serde_json::to_string(value)?;
        value.push('\n');
        self.stdin.write_all(value.as_bytes())?;
        debug!("Supervisor sent: {:?}", value);
        Ok(())
    }

    pub fn send_struct<T: serde::Serialize>(&mut self, value: T, tx: &EventSender) {
        let value = serde_json::to_value(value).expect("Failed to serialize reply");
        if let Err(_) = self.send(&value) {
            tx.send(Event2::Log(LogEvent::Error("Failed to send to child".into())));
        }
    }

    pub fn recv(&mut self) -> Result<Event> {
        let mut line = String::new();
        let len = self.stdout.read_line(&mut line)?;

        let event = serde_json::from_str(&line[..len])?;
        debug!("Supervisor received: {:?}", event);
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

    pub fn send_event_callback<T: EventWithCallback>(&mut self, event: T, tx: &EventSender)
        where <T as EventWithCallback>::Payload: serde::Serialize
    {
        let (tx2, rx2) = mpsc::channel();
        tx.send(event.with_callback(tx2));
        let reply = rx2.recv().unwrap();

        self.send_struct(reply, tx);
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
        let value = self.recv()?;
        let event = serde_json::from_value(value)?;
        Ok(event)
    }
}

impl Reporter for StdioReporter {
    fn send(&mut self, event: &Event) -> Result<()> {
        let mut event = serde_json::to_string(&event)?;
        event.push('\n');
        self.stdout.write_all(event.as_bytes())?;
        debug!("Reporter sent: {:?}", event);
        Ok(())
    }

    fn recv(&mut self) -> Result<serde_json::Value> {
        let mut line = String::new();
        let len = self.stdin.read_line(&mut line)?;

        let event = serde_json::from_str(&line[..len])?;
        debug!("Reporter received: {:?}", event);
        Ok(event)
    }
}

pub fn spawn_module(module: Module,
                    tx: &EventSender,
                    arg: serde_json::Value,
                    keyring: Vec<KeyRingEntry>,
                    verbose: u64,
                    has_stdin: bool,
                    proxy: Option<SocketAddr>,
                    options: HashMap<String, String>,
                    blobs: Vec<Blob>,
) -> Result<ExitEvent> {
    let dns_config = Resolver::from_system()?;

    let mut reader = if has_stdin {
        Some(BufReader::new(stdin()))
    } else {
        None
    };

    let mut supervisor = Supervisor::setup(&module)?;
    supervisor.send_start(&StartCommand::new(verbose, keyring, dns_config, proxy, options, module, arg, blobs))?;

    let exit = loop {
        match supervisor.recv()? {
            Event::Log(event) => tx.send(Event2::Log(event)),
            Event::Database(object) => supervisor.send_event_callback(object, &tx),
            Event::Stdio(object) => object.apply(&mut supervisor, tx, &mut reader),
            Event::Blob(blob) => supervisor.send_event_callback(blob, &tx),
            Event::Exit(event) => {
                if let ExitEvent::Err(err) = &event {
                    tx.send(Event2::Log(LogEvent::Error(err.clone())));
                }
                break event;
            },
        }
    };

    supervisor.wait()?;

    Ok(exit)
}

pub fn run_worker(geoip: MaxmindReader, asn: MaxmindReader, psl: PslReader) -> Result<()> {
    let mut reporter = StdioReporter::setup();
    let start = reporter.recv_start()?;

    let environment = Environment {
        verbose: start.verbose,
        keyring: start.keyring,
        dns_config: start.dns_config,
        proxy: start.proxy,
        options: start.options,
        blobs: start.blobs,
        psl,
        geoip,
        asn,
    };

    let mtx: Arc<Mutex<Box<Reporter>>> = Arc::new(Mutex::new(Box::new(reporter)));
    let result = start.module.run(environment,
                                  mtx.clone(),
                                  start.arg.into());
    let mut reporter = Arc::try_unwrap(mtx).expect("Failed to consume Arc")
                        .into_inner().expect("Failed to consume Mutex");

    let event = match result {
        Ok(_) => ExitEvent::Ok,
        Err(err) => ExitEvent::Err(err.to_string()),
    };
    reporter.send(&Event::Exit(event))?;

    Ok(())
}
