use crate::errors::*;

use crate::models::*;
use crate::shell::Readline;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::utils;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub enum Args {
    #[structopt(name="domain")]
    Domain(AddDomain),
    #[structopt(name="subdomain")]
    Subdomain(AddSubdomain),
    #[structopt(name="email")]
    Email(AddEmail),
    #[structopt(name="phonenumber")]
    PhoneNumber(AddPhoneNumber),
    #[structopt(name="device")]
    Device(AddDevice),
    #[structopt(name="network")]
    Network(AddNetwork),
}

#[derive(Debug, StructOpt)]
pub struct AddDomain {
    domain: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct AddSubdomain {
    subdomain: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct AddEmail {
    email: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct AddPhoneNumber {
    phonenumber: Option<String>,
    name: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct AddDevice {
    mac: Option<String>,
    name: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct AddNetwork {
    network: Option<String>,
    latitude: Option<f32>,
    longitude: Option<f32>,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    match args {
        Args::Domain(args) => add_domain(rl, args),
        Args::Subdomain(args) => add_subdomain(rl, args),
        Args::Email(args) => add_email(rl, args),
        Args::PhoneNumber(args) => add_phonenumber(rl, args),
        Args::Device(args) => add_device(rl, args),
        Args::Network(args) => add_network(rl, args),
    }
}

fn add_domain(rl: &mut Readline, args: AddDomain) -> Result<()> {
    let domain = match args.domain {
        Some(domain) => domain,
        _ => utils::question("Domain")?,
    };

    // ensure input is a valid domain
    let parsed_domain = rl.psl().parse_domain(&domain)
        .map_err(|e| format_err!("Failed to parse domain: {}", e))?;

    if Some(domain.as_str()) != parsed_domain.root() {
        bail!("This is not a valid domain, might be a subdomain or tld");
    }

    rl.db().insert_struct(NewDomain {
        value: &domain,
    })?;

    Ok(())
}

fn add_subdomain(rl: &mut Readline, args: AddSubdomain) -> Result<()> {
    let subdomain = match args.subdomain {
        Some(subdomain) => subdomain,
        _ => utils::question("Subdomain")?,
    };

    let dns_name = rl.psl().parse_dns_name(&subdomain)
        .map_err(|e| format_err!("Failed to parse dns_name: {}", e))?;

    let domain = dns_name.domain()
        .ok_or_else(|| format_err!("Dns Name seems invalid"))?
        .to_string();

    let domain_id = match rl.db().insert_struct(NewDomain {
        value: &domain,
    })? {
        Some((_, domain_id)) => domain_id,
        _ => bail!("Domain is out out of scope"),
    };

    rl.db().insert_struct(NewSubdomain {
        domain_id,
        value: &subdomain,
        resolvable: None,
    })?;

    Ok(())
}

fn add_email(rl: &mut Readline, args: AddEmail) -> Result<()> {
    let email = match args.email {
        Some(email) => email,
        _ => utils::question("Email")?,
    };

    rl.db().insert_struct(NewEmail {
        value: &email,
        valid: None,
    })?;

    Ok(())
}

fn add_phonenumber(rl: &mut Readline, args: AddPhoneNumber) -> Result<()> {
    let (phonenumber, name) = match args.phonenumber {
        Some(phonenumber) => {
            (phonenumber, args.name)
        },
        _ => {
            let phonenumber = utils::question("Phone Number")?;
            let name = utils::question_opt("Name")?;
            (phonenumber, name)
        },
    };

    rl.db().insert_struct(NewPhoneNumber {
        value: &phonenumber,
        name: name.as_ref(),
        valid: None,
        last_online: None,
        country: None,
        carrier: None,
        line: None,
        is_ported: None,
        last_ported: None,
        caller_name: None,
        caller_type: None,
    })?;

    Ok(())
}

fn add_device(rl: &mut Readline, args: AddDevice) -> Result<()> {
    let (mac, name) = match args.mac {
        Some(mac) => {
            (mac, args.name)
        },
        _ => {
            let mac = utils::question("Mac address")?;
            let name = utils::question_opt("Name")?;
            (mac, name)
        },
    };

    rl.db().insert_struct(NewDevice {
        value: &mac,
        name: name.as_ref(),
        hostname: None,
        vendor: None,
        last_seen: None,
    })?;

    Ok(())
}

fn add_network(rl: &mut Readline, args: AddNetwork) -> Result<()> {
    let (network, latitude, longitude) = match args.network {
        Some(network) => (network, args.latitude, args.longitude),
        _ => {
            let network = utils::question("Network")?;
            let latitude = utils::question_typed_opt("Latitude")?;
            let longitude = utils::question_typed_opt("Longitude")?;
            (network, latitude, longitude)
        }
    };

    rl.db().insert_struct(NewNetwork {
        value: &network,
        latitude,
        longitude,
    })?;

    Ok(())
}
