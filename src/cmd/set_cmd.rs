use crate::errors::*;

use crate::shell::Readline;
use structopt::StructOpt;
use structopt::clap::AppSettings;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    key: Option<String>,
    value: Option<String>,
}

// TODO: maybe introduce global settings
// TODO: maybe allow setting jobs here as well in addition to -j
pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;

    let options = rl.options_mut()
        .ok_or_else(|| format_err!("Module needs to be selected first"))?;

    match (args.key, args.value) {
        (None, None) => {
            for (key, value) in options.iter() {
                println!("{}={:?}", key, value);
            }
        },
        (Some(key), None) => {
            if let Some(value) = options.get(&key) {
                println!("{:?}", value);
            }
        },
        (Some(key), Some(value)) => {
            options.insert(key, value);
        },
        (None, Some(_)) => unreachable!(),
    }

    Ok(())
}
