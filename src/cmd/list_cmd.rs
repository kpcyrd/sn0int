use errors::*;

use colored::Colorize;
use shell::Readline;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub struct Args {
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let _args = Args::from_iter_safe(args)?;

    for module in rl.engine().list() {
        println!("{} ({})", module.canonical().green(),
                            module.version().yellow());
        println!("\t{}", module.description());
    }

    Ok(())
}
