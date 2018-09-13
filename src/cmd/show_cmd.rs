use errors::*;

use shell::Readline;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub enum Args {
    #[structopt(name="domains")]
    Domains,
    #[structopt(name="subdomains")]
    Subdomains,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    match args {
        Args::Domains => show_domains(rl),
        Args::Subdomains => show_subdomains(rl),
    }
}

fn show_domains(rl: &mut Readline) -> Result<()> {
    for domain in rl.db().list_domains()? {
        println!("{:#?}", domain);
    }

    Ok(())
}

fn show_subdomains(rl: &mut Readline) -> Result<()> {
    for subdomain in rl.db().list_subdomains()? {
        println!("{:#?}", subdomain);
    }

    Ok(())
}
