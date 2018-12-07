use crate::errors::*;

use crate::db;
use crate::shell::Readline;
use sn0int_common::metadata::Source;
use structopt::StructOpt;
use crate::term;
use crate::models::*;


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

    let source = source
        .ok_or_else(|| format_err!("Module doesn't have sources"))?;

    if args.filter.is_empty() {
        match source {
            Source::Domains => select::<Domain>(rl)?,
            Source::Subdomains => select::<Subdomain>(rl)?,
            Source::IpAddrs => select::<IpAddr>(rl)?,
            Source::Urls => select::<Url>(rl)?,
            Source::Emails => select::<Email>(rl)?,
        }
    } else {
        debug!("Setting filter to {:?}", args.filter);
        let filter = db::Filter::parse_optional(&args.filter)?;
        rl.set_target(Some(filter));
        term::info(&format!("{} entities selected", count_selected(rl, &source)?));
    }

    Ok(())
}

fn count_selected(rl: &mut Readline, source: &Source) -> Result<usize> {
    let db = rl.db();
    let filter = rl.scoped_targets();

    let num = match source {
        Source::Domains => db.filter::<Domain>(&filter)?.len(),
        Source::Subdomains => db.filter::<Subdomain>(&filter)?.len(),
        Source::IpAddrs => db.filter::<IpAddr>(&filter)?.len(),
        Source::Urls => db.filter::<Url>(&filter)?.len(),
        Source::Emails => db.filter::<Email>(&filter)?.len(),
    };
    Ok(num)
}

fn select<T: Model + Detailed>(rl: &mut Readline) -> Result<()> {
    let filter = rl.scoped_targets();

    for obj in rl.db().filter::<T>(&filter)? {
        println!("{}", obj.detailed(rl.db())?);
    }

    Ok(())
}
