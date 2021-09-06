use crate::args;
use colored::Colorize;
use crate::blobs::BlobStorage;
use crate::cmd::Cmd;
use crate::db::{ttl, Database};
use crate::errors::*;
use crate::models::*;
use crate::shell::{self, Shell};
use crate::workspaces;
use humansize::{FileSize, file_size_opts};
use separator::Separatable;
use serde::{Serialize, Deserialize};
use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(Debug, Clone, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Exclude blob storage
    #[structopt(short, long)]
    short: bool,
    /// Exclude categories that don't contain any structs
    #[structopt(short, long)]
    quiet: bool,
    /// Show workspace statistics in json
    #[structopt(short, long)]
    json: bool,
    /// Go through all workspaces
    #[structopt(short, long)]
    all: bool,
}

impl Args {
    fn show_amount(&self, label: &str, count: usize, amount: &str) {
        if self.quiet && count == 0 {
            return;
        }

        let label = format!("{:20}", label);
        let amount = format!("{:>20}", amount);

        if count > 0 {
            println!("{} {}", label.green(), amount.yellow());
        } else {
            println!("{} {}", label, amount);
        }
    }

    fn show_count(&self, label: &str, count: usize) {
        self.show_amount(label, count, &count.separated_string());
    }
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
            domains: count_models::<Domain>(db)?,
            subdomains: count_models::<Subdomain>(db)?,
            ipaddrs: count_models::<IpAddr>(db)?,
            urls: count_models::<Url>(db)?,
            emails: count_models::<Email>(db)?,
            phonenumbers: count_models::<PhoneNumber>(db)?,
            devices: count_models::<Device>(db)?,
            networks: count_models::<Network>(db)?,
            accounts: count_models::<Account>(db)?,
            breaches: count_models::<Breach>(db)?,
            images: count_models::<Image>(db)?,
            ports: count_models::<Port>(db)?,
            netblocks: count_models::<Netblock>(db)?,
            cryptoaddrs: count_models::<CryptoAddr>(db)?,
            activity: Activity::count(db)?,
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
    fn run(mut self, rl: &mut Shell) -> Result<()> {
        if self.all {
            self.all = false;
            let mut first = true;

            let config = rl.config();
            for ws in workspaces::list()? {
                if first {
                    first = false;
                } else if !self.json {
                    println!();
                }

                let mut rl = shell::init(&args::Args {
                    workspace: Some(ws),
                    subcommand: None,
                }, config, false)?;
                self.clone().run(&mut rl)?;
            }
        } else {
            ttl::reap_expired(rl)?;

            let db = rl.db();

            let workspace = rl.workspace();

            if !self.json {
                // print this early in case counting takes a while
                println!("{:>41}", workspace.bold());
            }

            let mut stats = Stats::count(workspace.into(), db)?;
            if !self.short {
                stats.add_blob_usage(rl.blobs())?;
            }

            if self.json {
                let stats = serde_json::to_string(&stats)?;
                println!("{}", stats);
            } else {
                self.show_count("domains", stats.domains);
                self.show_count("subdomains", stats.subdomains);
                self.show_count("ipaddrs", stats.ipaddrs);
                self.show_count("urls", stats.urls);
                self.show_count("emails", stats.emails);
                self.show_count("phonenumbers", stats.phonenumbers);
                self.show_count("devices", stats.devices);
                self.show_count("networks", stats.networks);
                self.show_count("accounts", stats.accounts);
                self.show_count("breaches", stats.breaches);
                self.show_count("images", stats.images);
                self.show_count("ports", stats.ports);
                self.show_count("netblocks", stats.netblocks);
                self.show_count("cryptoaddrs", stats.cryptoaddrs);
                self.show_count("activity", stats.activity);

                if let Some(blobs) = stats.blobs {
                    self.show_count("blobs", blobs.count);
                    self.show_amount("blobs (size)", blobs.total_size as usize, &blobs.total_human_size);
                }
            }
        }

        Ok(())
    }
}
