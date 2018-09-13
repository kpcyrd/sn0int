use errors::*;

use diesel::prelude::*;
use models::Domain;
use shell::Readline;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub enum Args {
    #[structopt(name="domains")]
    Domains,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    match args {
        Args::Domains => show_domains(rl),
    }
}

fn show_domains(rl: &mut Readline) -> Result<()> {
    use schema::domains::dsl::*;

    let results = domains.load::<Domain>(rl.db())?;

    for domain in results {
        println!("{:#?}", domain);
    }

    Ok(())
}
