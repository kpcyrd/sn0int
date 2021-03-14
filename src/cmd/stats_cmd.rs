use crate::errors::*;

use colored::Colorize;
use crate::cmd::Cmd;
use crate::db::Database;
use crate::db::ttl;
use crate::models::*;
use crate::shell::Shell;
use humansize::{FileSize, file_size_opts};
use separator::Separatable;
use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Exclude blob storage
    #[structopt(long)]
    short: bool,
}

fn show_amount(label: &str, count: usize, amount: &str) -> Result<()> {
    let label = format!("{:20}", label);
    let amount = format!("{:>20}", amount);

    if count > 0 {
        println!("{} {}", label.green(), amount.yellow());
    } else {
        println!("{} {}", label, amount);
    }

    Ok(())
}

fn show_count(label: &str, count: usize) -> Result<()> {
    show_amount(label, count, &count.separated_string())
}

fn count_models<T: Model>(label: &str, db: &Database) -> Result<()> {
    let query = db.list::<T>()?;
    let count = query.len();
    show_count(label, count)
}

impl Cmd for Args {
    #[inline]
    fn run(self, rl: &mut Shell) -> Result<()> {
        ttl::reap_expired(rl)?;

        let db = rl.db();
        println!("{:>41}", rl.workspace().bold());
        count_models::<Domain>("domain", &db)?;
        count_models::<Subdomain>("subdomains", &db)?;
        count_models::<IpAddr>("ipaddrs", &db)?;
        count_models::<Url>("urls", &db)?;
        count_models::<Email>("emails", &db)?;
        count_models::<PhoneNumber>("phonenumbers", &db)?;
        count_models::<Device>("devices", &db)?;
        count_models::<Network>("networks", &db)?;
        count_models::<Account>("accounts", &db)?;
        count_models::<Breach>("breaches", &db)?;
        count_models::<Image>("images", &db)?;
        count_models::<Port>("ports", &db)?;
        count_models::<Netblock>("netblocks", &db)?;
        count_models::<CryptoAddr>("cryptoaddrs", &db)?;
        show_count("activity", Activity::count(&db)?)?;

        if !self.short {
            let storage = rl.blobs();
            let blobs = storage.list()?;
            show_count("blobs", blobs.len())?;

            let mut total_size = 0;
            for blob in &blobs {
                total_size += storage.stat(blob)?;
            }
            let human_size = total_size.file_size(file_size_opts::CONVENTIONAL)
                .map_err(|e| format_err!("Failed to format size: {}", e))?;
            show_amount("blobs (size)", total_size as usize, &human_size)?;
        }

        Ok(())
    }
}
