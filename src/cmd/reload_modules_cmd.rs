use errors::*;

use shell::Readline;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub struct Args {
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let _args = Args::from_iter_safe(args)?;

    let current = rl.take_module()
                    .map(|m| m.canonical());

    rl.engine_mut().reload_modules()?;
    rl.reload_module_cache();

    if let Some(module) = current {
        if let Ok(module) = rl.engine().get(&module).map(|x| x.to_owned()) {
            rl.set_module(module);
        }
    }

    Ok(())
}
