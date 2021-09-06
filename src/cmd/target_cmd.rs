use crate::errors::*;

use crate::db;
use crate::shell::Shell;
use sn0int_common::metadata::Source;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::term;
use crate::models::*;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    // TODO: target -p # print current filter
    // TODO: target -c # clear filter

    filter: Vec<String>,
}

pub fn run(rl: &mut Shell, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;

    let source = rl.module()
        .ok_or_else(|| format_err!("No module selected"))
        .map(|x| x.source().clone())?;

    let source = source
        .ok_or_else(|| format_err!("Module doesn't have sources"))?;

    if args.filter.is_empty() {
        match source {
            Source::Domains => select::<Domain>(rl, None)?,
            Source::Subdomains => select::<Subdomain>(rl, None)?,
            Source::IpAddrs => select::<IpAddr>(rl, None)?,
            Source::Urls => select::<Url>(rl, None)?,
            Source::Emails => select::<Email>(rl, None)?,
            Source::PhoneNumbers => select::<PhoneNumber>(rl, None)?,
            Source::Networks => select::<Network>(rl, None)?,
            Source::Devices => select::<Device>(rl, None)?,
            Source::Accounts(service) => select::<Account>(rl, service.as_ref())?,
            Source::Breaches => select::<Breach>(rl, None)?,
            Source::Images => select::<Image>(rl, None)?,
            Source::Ports => select::<Port>(rl, None)?,
            Source::Netblocks => select::<Netblock>(rl, None)?,
            Source::CryptoAddrs(currency) => select::<CryptoAddr>(rl, currency.as_ref())?,
            Source::Notifications => bail!("Notifications can't be set as target"),
            Source::KeyRing(namespace) => {
                for key in rl.keyring().list_for(&namespace) {
                    println!("{}:{}", key.namespace, key.name);
                }
            },
        }
    } else {
        debug!("Setting filter to {:?}", args.filter);
        let filter = db::Filter::parse_optional(&args.filter)?;
        rl.set_target(Some(filter));
        term::info(&format!("{} entities selected", count_selected(rl, &source)?));
    }

    Ok(())
}

fn count_selected(rl: &mut Shell, source: &Source) -> Result<usize> {
    let db = rl.db();
    let filter = rl.scoped_targets();

    let num = match source {
        Source::Domains => db.filter::<Domain>(&filter)?.len(),
        Source::Subdomains => db.filter::<Subdomain>(&filter)?.len(),
        Source::IpAddrs => db.filter::<IpAddr>(&filter)?.len(),
        Source::Urls => db.filter::<Url>(&filter)?.len(),
        Source::Emails => db.filter::<Email>(&filter)?.len(),
        Source::PhoneNumbers => db.filter::<PhoneNumber>(&filter)?.len(),
        Source::Networks => db.filter::<Network>(&filter)?.len(),
        Source::Devices => db.filter::<Device>(&filter)?.len(),
        Source::Accounts(service) => db.filter_with_param::<Account>(&filter, service.as_ref())?.len(),
        Source::Breaches => db.filter::<Breach>(&filter)?.len(),
        Source::Images => db.filter::<Image>(&filter)?.len(),
        Source::Ports => db.filter::<Port>(&filter)?.len(),
        Source::Netblocks => db.filter::<Netblock>(&filter)?.len(),
        Source::CryptoAddrs(currency) => db.filter_with_param::<CryptoAddr>(&filter, currency.as_ref())?.len(),
        Source::Notifications => bail!("Notifications can't be set as target"),
        Source::KeyRing(namespace) => rl.keyring().list_for(namespace).len(),
    };
    Ok(num)
}

fn select<T: Model + Detailed>(rl: &mut Shell, param: Option<&String>) -> Result<()> {
    let filter = rl.scoped_targets();

    for obj in rl.db().filter_with_param::<T>(&filter, param)? {
        println!("{}", obj.detailed(rl.db())?);
    }

    Ok(())
}
