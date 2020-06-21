use crate::errors::*;

use crate::blobs::Blob;
use crate::cmd::Cmd;
use crate::gfx;
use crate::models::*;
use crate::shell::Shell;
use crate::term;
use crate::utils;
use ipnetwork;
use std::fs;
use std::net;
use std::net::SocketAddr;
use std::path::Path;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    #[structopt(subcommand)]
    subcommand: Target,
    /// Do not actually insert into database
    #[structopt(short = "n", long = "dry-run")]
    dry_run: bool,
}

#[derive(Debug, StructOpt)]
pub enum Target {
    /// Insert domain into the database
    #[structopt(name = "domain")]
    Domain(AddDomain),
    /// Insert subdomain into the database
    #[structopt(name = "subdomain")]
    Subdomain(AddSubdomain),
    /// Insert ip address into the database
    #[structopt(name = "ipaddr")]
    IpAddr(AddIpAddr),
    /// Insert url into the database
    #[structopt(name = "url")]
    Url(AddUrl),
    /// Insert email into the database
    #[structopt(name = "email")]
    Email(AddEmail),
    /// Insert phonenumber into the database
    #[structopt(name = "phonenumber")]
    PhoneNumber(AddPhoneNumber),
    /// Insert device into the database
    #[structopt(name = "device")]
    Device(AddDevice),
    /// Insert network into the database
    #[structopt(name = "network")]
    Network(AddNetwork),
    /// Insert account into the database
    #[structopt(name = "account")]
    Account(AddAccount),
    /// Insert breach into the database
    #[structopt(name = "breach")]
    Breach(AddBreach),
    /// Insert images into the database
    #[structopt(name = "image")]
    Image(AddImage),
    /// Insert ip network into the database
    #[structopt(name = "netblock")]
    Netblock(AddNetblock),
    /// Insert port into the database
    #[structopt(name = "port")]
    Port(AddPort),
    /// Insert a crypto currency address into the database
    #[structopt(name = "cryptoaddr")]
    CryptoAddr(AddCryptoAddr),
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        match self.subcommand {
            Target::Domain(args) => args.insert(rl, self.dry_run),
            Target::Subdomain(args) => args.insert(rl, self.dry_run),
            Target::IpAddr(args) => args.insert(rl, self.dry_run),
            Target::Url(args) => args.insert(rl, self.dry_run),
            Target::Email(args) => args.insert(rl, self.dry_run),
            Target::PhoneNumber(args) => args.insert(rl, self.dry_run),
            Target::Device(args) => args.insert(rl, self.dry_run),
            Target::Network(args) => args.insert(rl, self.dry_run),
            Target::Account(args) => args.insert(rl, self.dry_run),
            Target::Breach(args) => args.insert(rl, self.dry_run),
            Target::Image(args) => args.insert(rl, self.dry_run),
            Target::Netblock(args) => args.insert(rl, self.dry_run),
            Target::Port(args) => args.insert(rl, self.dry_run),
            Target::CryptoAddr(args) => args.insert(rl, self.dry_run),
        }
    }
}

trait IntoInsert: Sized {
    fn into_insert(self, rl: &mut Shell) -> Result<Insert>;

    fn insert(self, rl: &mut Shell, dry_run: bool) -> Result<()> {
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
    fn into_insert(self, rl: &mut Shell) -> Result<Insert> {
        let domain = match self.domain {
            Some(domain) => domain,
            _ => utils::question("Domain")?,
        };

        // ensure input is a valid domain
        let dns_name = rl
            .psl()?
            .parse_dns_name(&domain)
            .map_err(|e| format_err!("Failed to parse domain: {}", e))?;

        if dns_name.fulldomain.is_some() {
            bail!("Domain has an unexpected subdomain, add as a subdomain instead");
        }

        Ok(Insert::Domain(NewDomain {
            value: domain,
            unscoped: false,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddSubdomain {
    subdomain: Option<String>,
}

impl IntoInsert for AddSubdomain {
    fn into_insert(self, rl: &mut Shell) -> Result<Insert> {
        let subdomain = match self.subdomain {
            Some(subdomain) => subdomain,
            _ => utils::question("Subdomain")?,
        };

        let dns_name = rl
            .psl()?
            .parse_dns_name(&subdomain)
            .map_err(|e| format_err!("Failed to parse dns_name: {}", e))?;

        let domain_id = match rl.db().insert_struct(
            NewDomain {
                value: dns_name.root,
                unscoped: false,
            },
            true,
        )? {
            Some((_, domain_id)) => domain_id,
            _ => bail!("Domain is out out of scope"),
        };

        Ok(Insert::Subdomain(NewSubdomain {
            domain_id,
            value: subdomain,
            resolvable: None,
            unscoped: false,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddIpAddr {
    ipaddr: Option<net::IpAddr>,
}

impl IntoInsert for AddIpAddr {
    fn into_insert(self, _rl: &mut Shell) -> Result<Insert> {
        let ipaddr = match self.ipaddr {
            Some(ipaddr) => ipaddr,
            _ => {
                let ipaddr = utils::question("IP address")?;
                ipaddr.parse()?
            }
        };

        let family = match ipaddr {
            net::IpAddr::V4(_) => "4",
            net::IpAddr::V6(_) => "6",
        };

        Ok(Insert::IpAddr(NewIpAddr {
            family: family.to_string(),
            value: ipaddr.to_string(),
            continent: None,
            continent_code: None,
            country: None,
            country_code: None,
            city: None,
            latitude: None,
            longitude: None,
            asn: None,
            as_org: None,
            description: None,
            reverse_dns: None,
            unscoped: false,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddUrl {
    url: Option<String>,
}

impl IntoInsert for AddUrl {
    fn into_insert(self, rl: &mut Shell) -> Result<Insert> {
        let url = match self.url {
            Some(url) => url,
            _ => utils::question("URL")?,
        };

        let parts = url::Url::parse(&url)?;
        let subdomain = parts
            .domain()
            .ok_or_else(|| format_err!("url doesn't have a domain host"))?;

        let dns_name = rl
            .psl()?
            .parse_dns_name(&subdomain)
            .map_err(|e| format_err!("Failed to parse dns_name: {}", e))?;

        let domain_id = match rl.db().insert_struct(
            NewDomain {
                value: dns_name.root,
                unscoped: false,
            },
            true,
        )? {
            Some((_, domain_id)) => domain_id,
            _ => bail!("Domain is out out of scope"),
        };

        let subdomain_id = match rl.db().insert_struct(
            NewSubdomain {
                value: subdomain.to_string(),
                domain_id,
                resolvable: None,
                unscoped: false,
            },
            true,
        )? {
            Some((_, subdomain_id)) => subdomain_id,
            _ => bail!("Subdomain is out out of scope"),
        };

        Ok(Insert::Url(
            InsertUrl {
                subdomain_id,
                value: url,
                status: None,
                body: None,
                online: None,
                title: None,
                redirect: None,
            }
            .try_into_new()?,
        ))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddEmail {
    email: Option<String>,
}

impl IntoInsert for AddEmail {
    fn into_insert(self, _rl: &mut Shell) -> Result<Insert> {
        let email = match self.email {
            Some(email) => email,
            _ => utils::question("Email")?,
        };

        Ok(Insert::Email(NewEmail {
            value: email,
            displayname: None,
            valid: None,
            unscoped: false,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddPhoneNumber {
    phonenumber: Option<String>,
    name: Option<String>,
}

impl IntoInsert for AddPhoneNumber {
    fn into_insert(self, _rl: &mut Shell) -> Result<Insert> {
        let (phonenumber, name) = match self.phonenumber {
            Some(phonenumber) => (phonenumber, self.name),
            _ => {
                let phonenumber = utils::question("Phone Number")?;
                let name = utils::question_opt("Name")?;
                (phonenumber, name)
            }
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
            unscoped: false,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddDevice {
    mac: Option<String>,
    name: Option<String>,
}

impl IntoInsert for AddDevice {
    fn into_insert(self, _rl: &mut Shell) -> Result<Insert> {
        let (mac, name) = match self.mac {
            Some(mac) => (mac, self.name),
            _ => {
                let mac = utils::question("Mac address")?;
                let name = utils::question_opt("Name")?;
                (mac, name)
            }
        };

        Ok(Insert::Device(NewDevice {
            value: mac,
            name: name,
            hostname: None,
            vendor: None,
            last_seen: None,
            unscoped: false,
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
    fn into_insert(self, _rl: &mut Shell) -> Result<Insert> {
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
            description: None,
            unscoped: false,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddAccount {
    service: Option<String>,
    username: Option<String>,
}

impl IntoInsert for AddAccount {
    fn into_insert(self, _rl: &mut Shell) -> Result<Insert> {
        let service = if let Some(service) = self.service {
            service
        } else {
            utils::question("Service")?
        };

        if service.contains('/') {
            bail!("Service field can't contain `/`");
        }

        let username = if let Some(username) = self.username {
            username
        } else {
            utils::question("Username")?
        };

        let value = format!("{}/{}", service, username);

        Ok(Insert::Account(NewAccount {
            value: value,
            service: service,
            username: username,
            displayname: None,
            email: None,
            url: None,
            last_seen: None,
            birthday: None,
            phonenumber: None,
            profile_pic: None,
            unscoped: false,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddBreach {
    name: Option<String>,
}

impl IntoInsert for AddBreach {
    fn into_insert(self, _rl: &mut Shell) -> Result<Insert> {
        let name = match self.name {
            Some(name) => name,
            _ => {
                let name = utils::question("Name")?;
                name
            }
        };

        Ok(Insert::Breach(NewBreach {
            value: name,
            unscoped: false,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddImage {
    paths: Vec<String>,
}

impl IntoInsert for AddImage {
    fn into_insert(self, _rl: &mut Shell) -> Result<Insert> {
        unreachable!()
    }

    fn insert(self, rl: &mut Shell, dry_run: bool) -> Result<()> {
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

                        let err = err
                            .io_error()
                            .map(|err| err.to_string())
                            .unwrap_or_else(|| String::from("walkdir failed"));
                        term::error(&format!("Failed to access entry {:?}: {}", path, err));

                        continue;
                    }
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
                    }
                };

                // check if image
                let format = match gfx::guess_format(&data) {
                    Ok(format) => format.mime().to_string(),
                    _ => {
                        debug!("Probably not an image, skipping");
                        continue;
                    }
                };
                debug!("Detected image format: {:?}", format);

                let blob = Blob::create(data.into());
                if !dry_run {
                    rl.blobs().save(&blob)?;
                }
                let value = blob.id;

                let filename = path.file_name().to_string_lossy().to_string();

                term::info(&format!("{} {:?}", value, path.path()));

                if !dry_run {
                    rl.db().insert_generic(Insert::Image(NewImage {
                        value,

                        filename: Some(filename),
                        mime: Some(format),
                        width: None,
                        height: None,
                        created: None,

                        latitude: None,
                        longitude: None,

                        nudity: None,
                        ahash: None,
                        dhash: None,
                        phash: None,

                        unscoped: false,
                    }))?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, StructOpt)]
pub struct AddNetblock {
    ipnet: Option<ipnetwork::IpNetwork>,
}

impl IntoInsert for AddNetblock {
    fn into_insert(self, _rl: &mut Shell) -> Result<Insert> {
        let ipnet = match self.ipnet {
            Some(ipnet) => ipnet,
            _ => {
                let ipnet = utils::question("IP network")?;
                ipnet.parse()?
            }
        };

        let family = match ipnet {
            ipnetwork::IpNetwork::V4(_) => "4",
            ipnetwork::IpNetwork::V6(_) => "6",
        };

        Ok(Insert::Netblock(NewNetblock {
            family: family.to_string(),
            value: ipnet.to_string(),
            asn: None,
            as_org: None,
            description: None,
            unscoped: false,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddPort {
    protocol: Option<String>,
    addr: Option<SocketAddr>,
}

impl IntoInsert for AddPort {
    fn into_insert(self, rl: &mut Shell) -> Result<Insert> {
        let protocol = if let Some(protocol) = self.protocol {
            protocol
        } else {
            utils::question("Protocol (tcp/udp)")?
        };

        let addr = if let Some(addr) = self.addr {
            addr
        } else {
            let addr = utils::question("IP:Port")?;
            addr.parse()?
        };

        let family = match addr.ip() {
            net::IpAddr::V4(_) => "4",
            net::IpAddr::V6(_) => "6",
        };

        let ip_addr_id = match rl.db().insert_struct(
            NewIpAddr {
                family: family.to_string(),
                value: addr.ip().to_string(),
                continent: None,
                continent_code: None,
                country: None,
                country_code: None,
                city: None,
                latitude: None,
                longitude: None,
                asn: None,
                as_org: None,
                description: None,
                reverse_dns: None,
                unscoped: false,
            },
            true,
        )? {
            Some((_, ip_addr_id)) => ip_addr_id,
            _ => bail!("IpAddr is out out of scope"),
        };

        let value = format!("{}/{}", protocol, addr);

        Ok(Insert::Port(NewPort {
            ip_addr_id,
            value,
            ip_addr: addr.ip().to_string(),
            port: addr.port() as i32,
            protocol,
            status: None,

            banner: None,
            service: None,
            version: None,

            unscoped: false,
        }))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddCryptoAddr {
    address: Option<String>,
}

impl IntoInsert for AddCryptoAddr {
    fn into_insert(self, _rl: &mut Shell) -> Result<Insert> {
        let address = if let Some(address) = self.address {
            address
        } else {
            utils::question("Address")?
        };

        Ok(Insert::CryptoAddr(NewCryptoAddr {
            value: address,
            currency: None,
            denominator: None,
            balance: None,
            received: None,
            first_seen: None,
            last_withdrawal: None,
            unscoped: false,
            description: None,
        }))
    }
}
