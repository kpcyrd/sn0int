use errors::*;
use channel;
use chrootable_https::dns::DnsConfig;
use engine::{Environment, Module, Event, Reporter};
use geoip::{GeoIP, AsnDB};
use psl::Psl;
use serde_json;

use std::env;
use std::io::prelude::*;
use std::io::{self, BufReader, BufRead, Stdin, Stdout};
use std::result;
use std::sync::{mpsc, Arc, Mutex};
use std::process::{Command, Child, Stdio, ChildStdin, ChildStdout};


#[derive(Debug, Serialize, Deserialize)]
pub struct StartCommand {
    dns_config: DnsConfig,
    psl: String,
    module: Module,
    arg: serde_json::Value,
}

impl StartCommand {
    pub fn new(dns_config: DnsConfig, psl: String, module: Module, arg: serde_json::Value) -> StartCommand {
        // TODO: compress psl
        StartCommand {
            dns_config,
            psl,
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

    pub fn send_start(&mut self, dns_config: DnsConfig, psl: String, module: Module, arg: serde_json::Value) -> Result<()> {
        let start = serde_json::to_value(StartCommand::new(dns_config, psl, module, arg))?;
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

pub fn spawn_module(module: Module, tx: channel::Sender<(Event, Option<mpsc::Sender<result::Result<i32, String>>>)>, arg: serde_json::Value) -> Result<()> {
    let dns_config = DnsConfig::from_system()?;

    let psl = Psl::open_into_string()?;

    let mut supervisor = Supervisor::setup(&module)?;
    supervisor.send_start(dns_config, psl, module, arg)?;

    loop {
        match supervisor.recv()? {
            Event::Done => break,
            Event::Fatal(err) => {
                tx.send((Event::Fatal(err), None));
                break;
            },
            Event::Insert(object) => {
                let (tx2, rx2) = mpsc::channel();
                tx.send((Event::Insert(object), Some(tx2)));
                let reply = rx2.recv().unwrap();

                let value = serde_json::to_value(reply).expect("Failed to serialize reply");
                if let Err(_) = supervisor.send(&value) {
                    tx.send((Event::Error("Failed to send to child".into()), None));
                }
            },
            Event::Update(object) => {
                let (tx2, rx2) = mpsc::channel();
                tx.send((Event::Update(object), Some(tx2)));
                let reply = rx2.recv().unwrap();

                let value = serde_json::to_value(reply).expect("Failed to serialize reply");
                if let Err(_) = supervisor.send(&value) {
                    tx.send((Event::Error("Failed to send to child".into()), None));
                }
            },
            event => tx.send((event, None)),
        }
    }

    supervisor.wait()?;

    Ok(())
}

pub fn run_worker(geoip: GeoIP, asn: AsnDB) -> Result<()> {
    let mut reporter = StdioReporter::setup();
    let start = reporter.recv_start()?;

    let psl = Psl::from_str(&start.psl)
        .context("Failed to load public suffix list")?;
    let environment = Environment {
        dns_config: start.dns_config,
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

    if let Err(err) = result {
        reporter.send(&Event::Fatal(err.to_string()))?;
    } else {
        reporter.send(&Event::Done)?;
    }

    Ok(())
}
