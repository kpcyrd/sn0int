use crate::errors::*;
use crate::api::Client;
use crate::config::Config;
use crate::engine;
use crate::paths;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;

// 1 week
const UPDATE_INTERVAL: u64 = 3600 * 24 * 7;


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AutoUpdater {
    #[serde(default)]
    registry: Option<u64>,
    #[serde(default)]
    last_update: u64,
    #[serde(default)]
    outdated: u64,
}

impl AutoUpdater {
    fn path() -> Result<PathBuf> {
        let path = paths::data_dir()?;
        Ok(path.join("autoupdate.json"))
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    #[inline]
    pub fn outdated(&self) -> u64 {
        self.outdated
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
        fs::write(AutoUpdater::path()?, &config)?;
        Ok(())
    }

    pub fn all_updated(&mut self) {
        self.outdated = 0;
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

        let modules = modules.into_iter()
            .map(|x| x.clone())
            .collect();

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
        let latest = client.latest_release()?;

        if latest.time != self.registry {
            let mut outdated = 0;

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
                            debug!("Outdated: {}: {:?} -> {:?}", module.canonical(), installed, latest);
                            outdated += 1;
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
