use errors::*;

use shell::Readline;
use structopt::StructOpt;
use utils;


#[derive(Debug, StructOpt)]
pub enum Args {
    #[structopt(name="domain")]
    Domain(AddDomain),
    #[structopt(name="subdomain")]
    Subdomain(AddSubdomain),
    #[structopt(name="email")]
    Email(AddEmail),
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

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    match args {
        Args::Domain(args) => add_domain(rl, args),
        Args::Subdomain(args) => add_subdomain(rl, args),
        Args::Email(args) => add_email(rl, args),
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

    rl.db().insert_domain(&domain)?;

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

    rl.db().insert_subdomain(&subdomain, &domain)?;

    Ok(())
}

fn add_email(rl: &mut Readline, args: AddEmail) -> Result<()> {
    let email = match args.email {
        Some(email) => email,
        _ => utils::question("Email")?,
    };

    rl.db().insert_email(&email)?;

    Ok(())
}
