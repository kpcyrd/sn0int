use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::notify::NotificationConfig;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::net::SocketAddr;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub namespaces: HashMap<String, PathBuf>,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub notifications: HashMap<String, NotificationConfig>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let path = Config::path()?;
        Config::load_from(path)
    }

    pub fn path() -> Result<PathBuf> {
        let path = dirs_next::config_dir()
            .ok_or_else(|| format_err!("Failed to find config directory"))?;
        let path = path.join("sn0int.toml");
        Ok(path)
    }

    pub fn load_or_default() -> Result<Config> {
        let path = Config::path()?;
        if path.exists() {
            Config::load_from(&path)
        } else {
            Ok(Config::default())
        }
    }

    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Config> {
        let config = fs::read(&path)
            .context("Failed to read config file")?;

        let config = toml::from_slice(&config)?;

        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    #[serde(default="default_registry")]
    pub registry: String,
    #[serde(default, rename="no-autoupdate")]
    pub no_autoupdate: bool,
}

impl Default for CoreConfig {
    fn default() -> CoreConfig {
        CoreConfig {
            registry: default_registry(),
            no_autoupdate: false,
        }
    }
}

#[inline]
fn default_registry() -> String {
    String::from("https://sn0int.com")
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub proxy: Option<SocketAddr>,
    pub user_agent: Option<String>,
}
