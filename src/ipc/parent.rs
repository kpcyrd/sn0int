use crate::errors::*;
use crate::ipc::common::*;
use chrootable_https::dns::Resolver;
use crate::blobs::Blob;
use crate::engine::Module;
use crate::keyring::KeyRingEntry;
use crate::worker::{Event, Event2, LogEvent, ExitEvent, EventSender, EventWithCallback};
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::io::prelude::*;
use std::io::{BufReader, BufRead, stdin};
use std::net::SocketAddr;
use std::sync::mpsc;
use std::process::{Command, Child, Stdio, ChildStdin, ChildStdout};

pub struct IpcParent {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl IpcParent {
    pub fn setup(module: &Module) -> Result<IpcParent> {
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

        Ok(IpcParent {
            child,
            stdin,
            stdout,
        })
    }

    pub fn send_start(&mut self, start: &StartCommand) -> Result<()> {
        let start = serde_json::to_value(start)?;
        self.send(&start)?;
        Ok(())
    }

    pub fn send(&mut self, value: &serde_json::Value) -> Result<()> {
        let mut value = serde_json::to_string(value)?;
        value.push('\n');
        self.stdin.write_all(value.as_bytes())?;
        debug!("IpcParent sent: {:?}", value);
        Ok(())
    }

    pub fn send_struct<T: serde::Serialize>(&mut self, value: T, tx: &EventSender) {
        let value = serde_json::to_value(value).expect("Failed to serialize reply");
        if self.send(&value).is_err() {
            tx.send(Event2::Log(LogEvent::Error("Failed to send to child".into())));
        }
    }

    pub fn recv(&mut self) -> Result<Event> {
        let mut line = String::new();
        let len = self.stdout.read_line(&mut line)?;

        if len == 0 {
            bail!("Sandbox child has crashed");
        }

        let event = serde_json::from_str(&line[..len])?;
        debug!("IpcParent received: {:?}", event);
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

pub fn run(module: Module,
           tx: &EventSender,
           arg: serde_json::Value,
           keyring: Vec<KeyRingEntry>,
           verbose: u64,
           has_stdin: bool,
           proxy: Option<SocketAddr>,
           user_agent: Option<String>,
           options: HashMap<String, String>,
           blobs: Vec<Blob>,
) -> Result<ExitEvent> {
    let dns_config = Resolver::from_system_v4()?;

    let mut reader = if has_stdin {
        Some(BufReader::new(stdin()))
    } else {
        None
    };

    let mut ipc_parent = IpcParent::setup(&module)?;
    ipc_parent.send_start(&StartCommand::new(verbose, keyring, dns_config, proxy, user_agent, options, module, arg, blobs))?;

    let exit = loop {
        match ipc_parent.recv()? {
            Event::Log(event) => tx.send(Event2::Log(event)),
            Event::Database(object) => ipc_parent.send_event_callback(*object, tx),
            Event::Stdio(object) => object.apply(&mut ipc_parent, tx, &mut reader),
            Event::Ratelimit(req) => ipc_parent.send_event_callback(req, tx),
            Event::Blob(blob) => ipc_parent.send_event_callback(blob, tx),
            Event::Exit(event) => {
                if let ExitEvent::Err(err) = &event {
                    tx.send(Event2::Log(LogEvent::Error(err.clone())));
                }
                break event;
            },
        }
    };

    ipc_parent.wait()?;

    Ok(exit)
}
