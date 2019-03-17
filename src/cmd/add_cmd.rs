use crate::errors::*;

use crate::blobs::Blob;
use crate::cmd::Cmd;
use crate::models::*;
use crate::shell::Readline;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::utils;
use crate::term;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(subcommand)]
    subcommand: Target,
    /// Do not actually insert into database
    #[structopt(short="n", long="dry-run")]
    dry_run: bool,
}

#[derive(Debug, StructOpt)]
pub enum Target {
    /// Insert domain into the database
    #[structopt(name="domain")]
    Domain(AddDomain),
    /// Insert subdomain into the database
    #[structopt(name="subdomain")]
    Subdomain(AddSubdomain),
    /// Insert email into the database
    #[structopt(name="email")]
    Email(AddEmail),
    /// Insert phonenumber into the database
    #[structopt(name="phonenumber")]
    PhoneNumber(AddPhoneNumber),
    /// Insert device into the database
    #[structopt(name="device")]
    Device(AddDevice),
    /// Insert network into the database
    #[structopt(name="network")]
    Network(AddNetwork),
    /// Insert account into the database
    #[structopt(name="account")]
    Account(AddAccount),
    /// Insert breach into the database
    #[structopt(name="breach")]
    Breach(AddBreach),
    /// Insert images into the database
    #[structopt(name="image")]
    Image(AddImage),
}

impl Cmd for Args {
    fn run(self, rl: &mut Readline) -> Result<()> {
        match self.subcommand {
            Target::Domain(args) => args.insert(rl, self.dry_run),
            Target::Subdomain(args) => args.insert(rl, self.dry_run),
            Target::Email(args) => args.insert(rl, self.dry_run),
            Target::PhoneNumber(args) => args.insert(rl, self.dry_run),
            Target::Device(args) => args.insert(rl, self.dry_run),
            Target::Network(args) => args.insert(rl, self.dry_run),
            Target::Account(args) => args.insert(rl, self.dry_run),
            Target::Breach(args) => args.insert(rl, self.dry_run),
            Target::Image(args) => args.insert(rl, self.dry_run),
        }
    }
}

#[inline]
pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    Args::run_str(rl, args)
}

trait IntoInsert: Sized {
    fn into_insert(self, rl: &Readline) -> Result<Insert>;

    fn insert(self, rl: &Readline, dry_run: bool) -> Result<()> {
        let insert = self.into_insert(rl)?;
        if !dry_run {
            rl.db().insert_generic(insert)?;
        }
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
pub struct AddDomain {
    domain: Option<String>,
}

impl IntoInsert for AddDomain {
    fn into_insert(self, rl: &Readline) -> Result<Insert> {
        let domain = match self.domain {
            Some(domain) => domain,
            _ => utils::question("Domain")?,
        };

        // ensure input is a valid domain
        let parsed_domain = rl.psl().parse_domain(&domain)
            .map_err(|e| format_err!("Failed to parse domain: {}", e))?;

        if Some(domain.as_str()) != parsed_domain.root() {
            bail!("This is not a valid domain, might be a subdomain or tld");
        }

        Ok(Insert::Domain(NewDomain {
            value: domain,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddSubdomain {
    subdomain: Option<String>,
}

impl IntoInsert for AddSubdomain {
    fn into_insert(self, rl: &Readline) -> Result<Insert> {
        let subdomain = match self.subdomain {
            Some(subdomain) => subdomain,
            _ => utils::question("Subdomain")?,
        };

        let dns_name = rl.psl().parse_dns_name(&subdomain)
            .map_err(|e| format_err!("Failed to parse dns_name: {}", e))?;

        let domain = dns_name.domain()
            .ok_or_else(|| format_err!("Dns Name seems invalid"))?
            .to_string();

        let domain_id = match rl.db().insert_struct(NewDomain {
            value: domain,
        })? {
            Some((_, domain_id)) => domain_id,
            _ => bail!("Domain is out out of scope"),
        };

        Ok(Insert::Subdomain(NewSubdomain {
            domain_id,
            value: subdomain,
            resolvable: None,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddEmail {
    email: Option<String>,
}

impl IntoInsert for AddEmail {
    fn into_insert(self, _rl: &Readline) -> Result<Insert> {
        let email = match self.email {
            Some(email) => email,
            _ => utils::question("Email")?,
        };

        Ok(Insert::Email(NewEmail {
            value: email,
            valid: None,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddPhoneNumber {
    phonenumber: Option<String>,
    name: Option<String>,
}

impl IntoInsert for AddPhoneNumber {
    fn into_insert(self, _rl: &Readline) -> Result<Insert> {
        let (phonenumber, name) = match self.phonenumber {
            Some(phonenumber) => (phonenumber, self.name),
            _ => {
                let phonenumber = utils::question("Phone Number")?;
                let name = utils::question_opt("Name")?;
                (phonenumber, name)
            },
        };

        Ok(Insert::PhoneNumber(NewPhoneNumber {
            value: phonenumber,
            name: name,
            valid: None,
            last_online: None,
            country: None,
            carrier: None,
            line: None,
            is_ported: None,
            last_ported: None,
            caller_name: None,
            caller_type: None,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddDevice {
    mac: Option<String>,
    name: Option<String>,
}

impl IntoInsert for AddDevice {
    fn into_insert(self, _rl: &Readline) -> Result<Insert> {
        let (mac, name) = match self.mac {
            Some(mac) => {
                (mac, self.name)
            },
            _ => {
                let mac = utils::question("Mac address")?;
                let name = utils::question_opt("Name")?;
                (mac, name)
            },
        };

        Ok(Insert::Device(NewDevice {
            value: mac,
            name: name,
            hostname: None,
            vendor: None,
            last_seen: None,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddNetwork {
    network: Option<String>,
    latitude: Option<f32>,
    longitude: Option<f32>,
}

impl IntoInsert for AddNetwork {
    fn into_insert(self, _rl: &Readline) -> Result<Insert> {
        let (network, latitude, longitude) = match self.network {
            Some(network) => (network, self.latitude, self.longitude),
            _ => {
                let network = utils::question("Network")?;
                let latitude = utils::question_typed_opt("Latitude")?;
                let longitude = utils::question_typed_opt("Longitude")?;
                (network, latitude, longitude)
            }
        };

        Ok(Insert::Network(NewNetwork {
            value: network,
            latitude,
            longitude,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddAccount {
    service: Option<String>,
    username: Option<String>,
}

impl IntoInsert for AddAccount {
    fn into_insert(self, _rl: &Readline) -> Result<Insert> {
        let (service, username) = match (self.service, self.username) {
            (Some(service), Some(username)) => (service, username),
            _ => {
                let service = utils::question("Service")?;
                let username = utils::question("Username")?;

                (service, username)
            },
        };

        if service.contains('/') {
            // TODO: avoid duplication
            bail!("Service field can't contain `/`");
        }

        let value = format!("{}/{}", service, username);

        Ok(Insert::Account(NewAccount {
            value: value,
            service: service,
            username: username,
            displayname: None,
            email: None,
            url: None,
            last_seen: None,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddBreach {
    name: Option<String>,
}

impl IntoInsert for AddBreach {
    fn into_insert(self, _rl: &Readline) -> Result<Insert> {
        let name = match self.name {
            Some(name) => name,
            _ => {
                let name = utils::question("Name")?;
                name
            }
        };

        Ok(Insert::Breach(NewBreach {
            value: name,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddImage {
    paths: Vec<String>,
}

impl IntoInsert for AddImage {
    fn into_insert(self, _rl: &Readline) -> Result<Insert> {
        unreachable!()
    }

    fn insert(self, rl: &Readline, dry_run: bool) -> Result<()> {
        let paths = if self.paths.is_empty() {
            let path = utils::question("Path")?;
            vec![path]
        } else {
            self.paths
        };

        for path in paths {
            for path in WalkDir::new(path) {
                let path = match path {
                    Ok(path) => path,
                    Err(err) => {
                        let path = err.path().unwrap_or(Path::new("")).display();

                        let err = err.io_error()
                            .map(|err| err.to_string())
                            .unwrap_or_else(|| String::from("walkdir failed"));
                        term::error(&format!("Failed to access entry {:?}: {}", path, err));

                        continue;
                    },
                };

                if path.file_type().is_dir() {
                    debug!("Traversing into directory: {:?}", path);
                    continue;
                }

                debug!("Testing: {:?}", path.path());

                let data = match fs::read(path.path()) {
                    Ok(data) => data,
                    Err(err) => {
                        term::error(&format!("Failed to read {:?}: {}", path, err));
                        continue;
                    },
                };

                // check if image
                if let Ok(format) = image::guess_format(&data) {
                    debug!("Detected image format: {:?}", format);
                } else {
                    debug!("Probably not an image, skipping");
                    continue;
                }

                let blob = Blob::create(data.into());
                rl.blobs().save(&blob)?;
                let value = blob.id;

                let filename = path.file_name()
                    .to_string_lossy()
                    .to_string();

                term::info(&format!("{} {:?}", value, path.path()));

                if !dry_run {
                    rl.db().insert_generic(Insert::Image(NewImage {
                        value,

                        filename: Some(filename),
                        mime: None,
                        width: None,
                        height: None,
                        created: None,

                        latitude: None,
                        longitude: None,

                        nudity: None,
                        ahash: None,
                        dhash: None,
                        phash: None,
                    }))?;
                }
            }
        }

        Ok(())
    }
}
