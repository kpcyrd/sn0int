use crate::errors::*;
use crate::ipc::common::*;
use crate::engine::Environment;
use crate::geoip::MaxmindReader;
use crate::psl::PslReader;
use crate::worker::Event;
use std::fmt::Debug;
use std::io::prelude::*;
use std::io::{self, Stdin, Stdout};
use std::sync::{Arc, Mutex};

pub trait IpcChild: Debug {
    fn send(&mut self, event: &Event) -> Result<()>;

    fn recv(&mut self) -> Result<serde_json::Value>;
}

#[derive(Debug)]
pub struct StdioIpcChild {
    stdin: Stdin,
    stdout: Stdout,
}

impl StdioIpcChild {
    pub fn setup() -> StdioIpcChild {
        let stdin = io::stdin();
        let stdout = io::stdout();

        StdioIpcChild {
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

impl IpcChild for StdioIpcChild {
    fn send(&mut self, event: &Event) -> Result<()> {
        let mut event = serde_json::to_string(&event)?;
        event.push('\n');
        debug!("IpcChild sends: {:?}", event);
        self.stdout.write_all(event.as_bytes())?;
        Ok(())
    }

    fn recv(&mut self) -> Result<serde_json::Value> {
        let mut line = String::new();
        let len = self.stdin.read_line(&mut line)?;

        let event = serde_json::from_str(&line[..len])?;
        debug!("IpcChild received: {:?}", event);
        Ok(event)
    }
}

#[derive(Debug)]
pub struct DummyIpcChild;

impl DummyIpcChild {
    pub fn create() -> Arc<Mutex<Box<dyn IpcChild>>> {
        Arc::new(Mutex::new(Box::new(DummyIpcChild)))
    }
}

impl IpcChild for DummyIpcChild {
    fn send(&mut self, _event: &Event) -> Result<()> {
        Ok(())
    }

    fn recv(&mut self) -> Result<serde_json::Value> {
        unimplemented!("DummyIpcChild::recv doesn't exist")
    }
}

pub fn run(geoip: Option<MaxmindReader>, asn: Option<MaxmindReader>, psl: PslReader) -> Result<()> {
    let mut ipc_child = StdioIpcChild::setup();
    let start = ipc_child.recv_start()?;

    let environment = Environment {
        verbose: start.verbose,
        keyring: start.keyring,
        dns_config: start.dns_config,
        proxy: start.proxy,
        user_agent: start.user_agent,
        options: start.options,
        blobs: start.blobs,
        psl,
        geoip,
        asn,
    };

    let mtx: Arc<Mutex<Box<dyn IpcChild>>> = Arc::new(Mutex::new(Box::new(ipc_child)));
    let result = start.module.run(environment,
                                  mtx.clone(),
                                  start.arg.into());
    let mut ipc_child = Arc::try_unwrap(mtx).expect("Failed to consume Arc")
                        .into_inner().expect("Failed to consume Mutex");

    let event = result.into();
    ipc_child.send(&Event::Exit(event))?;

    Ok(())
}
