use errors::*;

use shell::Readline;
use structopt::StructOpt;
use publicsuffix;
use utils;


#[derive(Debug, StructOpt)]
pub enum Args {
    #[structopt(name="domain")]
    Domain(AddDomain),
    #[structopt(name="subdomain")]
    Subdomain(AddSubdomain),
}

#[derive(Debug, StructOpt)]
pub struct AddDomain {
    domain: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct AddSubdomain {
    subdomain: Option<String>,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    match args {
        Args::Domain(args) => add_domain(rl, args),
        Args::Subdomain(args) => add_subdomain(rl, args),
    }
}

fn add_domain(rl: &mut Readline, args: AddDomain) -> Result<()> {
    let domain = match args.domain {
        Some(domain) => domain,
        _ => utils::question("Domain")?,
    };

    // TODO: download this list automatically
    let list = publicsuffix::List::from_path("public_suffix_list.dat")
        .map_err(|e| format_err!("Failed to load public suffix list: {}", e))?;

    // ensure input is a valid domain
    let parsed_domain = list.parse_domain(&domain)
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

    // TODO: download this list automatically
    let list = publicsuffix::List::from_path("public_suffix_list.dat")
        .map_err(|e| format_err!("Failed to load public suffix list: {}", e))?;

    let dns_name = list.parse_dns_name(&subdomain)
        .map_err(|e| format_err!("Failed to parse dns_name: {}", e))?;

    let domain = dns_name.domain()
        .ok_or_else(|| format_err!("Dns Name seems invalid"))?
        .to_string();

    rl.db().insert_subdomain(&subdomain, &domain)?;

    Ok(())
}
