use clap::Parser;
use clap::ValueEnum;
use crate::blobs::Blob;
use crate::cmd::Cmd;
use crate::db::ttl;
use crate::errors::*;
use crate::models::*;
use crate::shell::Shell;
use serde::{Serialize, Deserialize};
use std::io::{self, Write};
use strum_macros::{EnumString, IntoStaticStr};

#[derive(Debug, Parser)]
pub struct Args {
    /// Specify the export format
    #[arg(short = 'f', long="format", value_enum)]
    format: Format,
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        ttl::reap_expired(rl)?;
        match self.format {
            Format::Json => export::<JsonFormat>(rl),
            Format::JsonBlobs => export::<JsonBlobsFormat>(rl),
        }
    }
}

fn export<T: ExportFormat + Serialize>(rl: &mut Shell) -> Result<()> {
    let export = T::load(rl)?;
    let mut stdout = io::stdout();
    serde_json::to_writer(&mut stdout, &export)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
#[derive(EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab_case")]
pub enum Format {
    Json,
    JsonBlobs,
}

trait ExportFormat {
    fn load(rl: &mut Shell) -> Result<Box<Self>>;
}

#[derive(Serialize, Deserialize)]
struct JsonFormat {
    accounts: Vec<Account>,
    breaches: Vec<Breach>,
    breach_emails: Vec<BreachEmail>,
    devices: Vec<Device>,
    domains: Vec<Domain>,
    emails: Vec<Email>,
    images: Vec<Image>,
    ipaddrs: Vec<IpAddr>,
    netblocks: Vec<Netblock>,
    networks: Vec<Netblock>,
    network_devices: Vec<NetworkDevice>,
    phonenumbers: Vec<PhoneNumber>,
    ports: Vec<Port>,
    subdomains: Vec<Subdomain>,
    subdomain_ipaddrs: Vec<SubdomainIpAddr>,
    urls: Vec<Url>,
}

impl ExportFormat for JsonFormat {
    fn load(rl: &mut Shell) -> Result<Box<JsonFormat>> {
        let db = rl.db();

        Ok(Box::new(JsonFormat {
            accounts: Account::list(db)?,
            breaches: Breach::list(db)?,
            breach_emails: BreachEmail::list(db)?,
            devices: Device::list(db)?,
            domains: Domain::list(db)?,
            emails: Email::list(db)?,
            images: Image::list(db)?,
            ipaddrs: IpAddr::list(db)?,
            netblocks: Netblock::list(db)?,
            networks: Netblock::list(db)?,
            network_devices: NetworkDevice::list(db)?,
            phonenumbers: PhoneNumber::list(db)?,
            ports: Port::list(db)?,
            subdomains: Subdomain::list(db)?,
            subdomain_ipaddrs: SubdomainIpAddr::list(db)?,
            urls: Url::list(db)?,
        }))
    }
}

#[derive(Serialize, Deserialize)]
struct JsonBlobsFormat {
    models: JsonFormat,
    blobs: Vec<Blob>,
}

impl ExportFormat for JsonBlobsFormat {
    fn load(rl: &mut Shell) -> Result<Box<JsonBlobsFormat>> {
        let models = *JsonFormat::load(rl)?;
        let storage = rl.blobs();
        let blobs = storage.list()?
            .into_iter()
            .map(|id| storage.load(&id))
            .collect::<Result<Vec<_>>>()?;
        Ok(Box::new(JsonBlobsFormat {
            models,
            blobs,
        }))
    }
}
