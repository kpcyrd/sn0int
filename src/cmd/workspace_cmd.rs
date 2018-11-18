use errors::*;

use db::Database;
use shell::Readline;
use structopt::StructOpt;
use workspaces;


#[derive(Debug, StructOpt)]
pub struct Args {
    workspace: Option<String>,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;

    if let Some(workspace) = args.workspace {
        let db = Database::establish(workspace)?;
        rl.set_db(db);
    } else {
        for x in workspaces::list()? {
            println!("{:?}", x);
        }
    }

    Ok(())
}
