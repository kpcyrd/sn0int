use errors::*;

use db;
use shell::Readline;
use sn0int_common::metadata::Source;
use structopt::StructOpt;
use models::*;


#[derive(Debug, StructOpt)]
pub struct Args {
    // TODO: target -p # print current filter
    // TODO: target -c # clear filter

    filter: Vec<String>,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;

    let source = rl.module()
        .ok_or_else(|| format_err!("No module selected"))
        .map(|x| x.source().clone())?;

    if args.filter.is_empty() {
        match source {
            Some(Source::Domains) => select::<Domain>(rl)?,
            Some(Source::Subdomains) => select::<Subdomain>(rl)?,
            Some(Source::IpAddrs) => select::<IpAddr>(rl)?,
            Some(Source::Urls) => select::<Url>(rl)?,
            Some(Source::Emails) => select::<Email>(rl)?,
            None => bail!("Module doesn't have sources"),
        }
    } else {
        debug!("Setting filter to {:?}", args.filter);
        let filter = db::Filter::parse_optional(&args.filter)?;
        rl.set_target(Some(filter));
    }

    Ok(())
}

fn select<T: Model + Detailed>(rl: &mut Readline) -> Result<()> {
    let filter = rl.scoped_targets();

    for obj in rl.db().filter::<T>(&filter)? {
        println!("{}", obj.detailed(rl.db())?);
    }

    Ok(())
}
