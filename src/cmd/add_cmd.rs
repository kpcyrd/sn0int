use errors::*;

use diesel;
use diesel::prelude::*;
use models::NewDomain;
use schema::domains;
use shell::Readline;
use structopt::StructOpt;
use utils;


#[derive(Debug, StructOpt)]
pub enum Args {
    #[structopt(name="domain")]
    Domain(AddDomain),
}

#[derive(Debug, StructOpt)]
pub struct AddDomain {
    domain: Option<String>,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    match args {
        Args::Domain(args) => add_domain(rl, args),
    }
}

fn add_domain(rl: &mut Readline, args: AddDomain) -> Result<()> {
    let domain = match args.domain {
        Some(domain) => domain,
        _ => utils::question("Domain")?,
    };

    // TODO: ensure input is valid domain

    let new_domain = NewDomain {
        value: &domain,
    };

    diesel::insert_into(domains::table)
        .values(&new_domain)
        .execute(rl.db())?;

    Ok(())
}
