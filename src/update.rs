use crate::errors::*;
use crate::api::Client;
use crate::config::Config;
use crate::engine;
use crate::paths;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;

// 1 day
const UPDATE_INTERVAL: u64 = 3600 * 24;


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AutoUpdater {
    #[serde(default)]
    registry: Option<u64>,
    #[serde(default)]
    last_update: u64,
    #[serde(default)]
    outdated: HashSet<String>,
}

impl AutoUpdater {
    #[inline]
    fn path() -> Result<PathBuf> {
        let path = paths::sn0int_dir()?;
        Ok(path.join("autoupdate.json"))
    }

    #[inline]
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    #[inline]
    pub fn outdated(&self) -> usize {
        self.outdated.len()
    }

    #[inline]
    pub fn updated(&mut self, module: &str) {
        self.outdated.remove(module);
    }

    #[inline]
    pub fn is_outdated(&self, module: &str) -> bool {
        self.outdated.contains(module)
    }

    pub fn load() -> Result<AutoUpdater> {
        let config = fs::read(AutoUpdater::path()?)
            .unwrap_or_else(|_| b"{}".to_vec());

        let config = serde_json::from_slice(&config)
            .unwrap_or_else(|_| AutoUpdater::default());

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config = serde_json::to_string(&self)?;
        fs::write(AutoUpdater::path()?, config)
            .context("Failed to write auto-update state")?;
        Ok(())
    }

    pub fn check_background(mut self, config: &Config, modules: Vec<&engine::Module>) {
        if config.core.no_autoupdate {
            debug!("Auto update has been disabled, skipping");
            return;
        }

        if self.last_update + UPDATE_INTERVAL >= AutoUpdater::now() {
            debug!("Auto update timer hasn't expired yet");
            return;
        }

        let modules = modules.into_iter().cloned().collect();

        debug!("Checking for outdated modules in the background");
        match Client::new(config) {
            Ok(client) => {
                thread::spawn(move || {
                    if let Err(err) = self.check_updates(client, modules) {
                        error!("AutoUpdater failed: {}", err);
                    } else {
                        debug!("AutoUpdater finished");
                    }
                });
            },
            Err(err) => error!("Failed to create client: {}", err),
        }
    }

    pub fn check_updates(&mut self, client: Client, modules: Vec<engine::Module>) -> Result<()> {
        let latest = client.latest_release()
            .context("Failed to get latest release timestamp")?;

        if latest.time != self.registry {
            let mut outdated = HashSet::new();

            for module in modules {
                if module.is_private() {
                    debug!("{} is a private module, skipping", module.canonical());
                    continue;
                }

                let installed = module.version();
                if let Ok(infos) = client.query_module(&module.id()) {
                    debug!("Latest version: {:?}", infos);

                    if let Some(latest) = infos.latest {
                        if installed != latest {
                            let canonical = module.canonical();
                            debug!("Outdated: {}: {:?} -> {:?}", canonical, installed, latest);
                            outdated.insert(canonical);
                        }
                    }
                }
            }

            self.outdated = outdated;
        }

        self.registry = latest.time;
        self.last_update = AutoUpdater::now();
        self.save()?;

        Ok(())
    }
}
