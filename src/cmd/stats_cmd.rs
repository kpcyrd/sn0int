use crate::errors::*;

use colored::Colorize;
use crate::blobs::BlobStorage;
use crate::cmd::Cmd;
use crate::db::Database;
use crate::db::ttl;
use crate::models::*;
use crate::shell::Shell;
use humansize::{FileSize, file_size_opts};
use separator::Separatable;
use serde::{Serialize, Deserialize};
use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Exclude blob storage
    #[structopt(long)]
    short: bool,
    /// Show workspace statistics in json
    #[structopt(long)]
    json: bool,
}

fn show_amount(label: &str, count: usize, amount: &str) {
    let label = format!("{:20}", label);
    let amount = format!("{:>20}", amount);

    if count > 0 {
        println!("{} {}", label.green(), amount.yellow());
    } else {
        println!("{} {}", label, amount);
    }
}

fn show_count(label: &str, count: usize) {
    show_amount(label, count, &count.separated_string());
}

fn count_models<T: Model>(db: &Database) -> Result<usize> {
    let query = db.list::<T>()?;
    let count = query.len();
    Ok(count)
}

#[derive(Debug, Serialize, Deserialize)]
struct Stats {
    workspace: String,
    domains: usize,
    subdomains: usize,
    ipaddrs: usize,
    urls: usize,
    emails: usize,
    phonenumbers: usize,
    devices: usize,
    networks: usize,
    accounts: usize,
    breaches: usize,
    images: usize,
    ports: usize,
    netblocks: usize,
    cryptoaddrs: usize,
    activity: usize,
    blobs: Option<BlobStats>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BlobStats {
    count: usize,
    total_size: u64,
    total_human_size: String,
}

impl Stats {
    fn count(workspace: String, db: &Database) -> Result<Stats> {
        Ok(Stats {
            workspace,
            domains: count_models::<Domain>(&db)?,
            subdomains: count_models::<Subdomain>(&db)?,
            ipaddrs: count_models::<IpAddr>(&db)?,
            urls: count_models::<Url>(&db)?,
            emails: count_models::<Email>(&db)?,
            phonenumbers: count_models::<PhoneNumber>(&db)?,
            devices: count_models::<Device>(&db)?,
            networks: count_models::<Network>(&db)?,
            accounts: count_models::<Account>(&db)?,
            breaches: count_models::<Breach>(&db)?,
            images: count_models::<Image>(&db)?,
            ports: count_models::<Port>(&db)?,
            netblocks: count_models::<Netblock>(&db)?,
            cryptoaddrs: count_models::<CryptoAddr>(&db)?,
            activity: Activity::count(&db)?,
            blobs: None,
        })
    }

    fn add_blob_usage(&mut self, storage: &BlobStorage) -> Result<()> {
        let blobs = storage.list()?;

        let mut total_size = 0;
        for blob in &blobs {
            total_size += storage.stat(blob)?;
        }

        let total_human_size = total_size.file_size(file_size_opts::CONVENTIONAL)
            .map_err(|e| format_err!("Failed to format size: {}", e))?;

        self.blobs = Some(BlobStats {
            count: blobs.len(),
            total_size,
            total_human_size,
        });

        Ok(())
    }
}

impl Cmd for Args {
    #[inline]
    fn run(self, rl: &mut Shell) -> Result<()> {
        ttl::reap_expired(rl)?;

        let db = rl.db();

        let mut stats = Stats::count(rl.workspace().into(), &db)?;
        if !self.short {
            stats.add_blob_usage(rl.blobs())?;
        }

        if self.json {
            let stats = serde_json::to_string(&stats)?;
            println!("{}", stats);
        } else {
            println!("{:>41}", stats.workspace.bold());
            show_count("domains", stats.domains);
            show_count("subdomains", stats.subdomains);
            show_count("ipaddrs", stats.ipaddrs);
            show_count("urls", stats.urls);
            show_count("emails", stats.emails);
            show_count("phonenumbers", stats.phonenumbers);
            show_count("devices", stats.devices);
            show_count("networks", stats.networks);
            show_count("accounts", stats.accounts);
            show_count("breaches", stats.breaches);
            show_count("images", stats.images);
            show_count("ports", stats.ports);
            show_count("netblocks", stats.netblocks);
            show_count("cryptoaddrs", stats.cryptoaddrs);
            show_count("activity", stats.activity);

            if let Some(blobs) = stats.blobs {
                show_count("blobs", blobs.count);
                show_amount("blobs (size)", blobs.total_size as usize, &blobs.total_human_size);
            }
        }

        Ok(())
    }
}
