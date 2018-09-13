use errors::*;

use db::Database;
use shell::Readline;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub struct Args {
    workspace: String,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    let db = Database::establish(args.workspace)?;
    rl.set_db(db);
    Ok(())
}
