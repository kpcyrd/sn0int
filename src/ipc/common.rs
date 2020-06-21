use crate::blobs::Blob;
use crate::engine::Module;
use crate::keyring::KeyRingEntry;
use chrootable_https::dns::Resolver;
use serde_json;

use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize)]
pub struct StartCommand {
    pub verbose: u64,
    pub keyring: Vec<KeyRingEntry>,
    pub dns_config: Resolver,
    pub proxy: Option<SocketAddr>,
    pub options: HashMap<String, String>,
    pub module: Module,
    pub arg: serde_json::Value,
    pub blobs: Vec<Blob>,
}

impl StartCommand {
    pub fn new(
        verbose: u64,
        keyring: Vec<KeyRingEntry>,
        dns_config: Resolver,
        proxy: Option<SocketAddr>,
        options: HashMap<String, String>,
        module: Module,
        arg: serde_json::Value,
        blobs: Vec<Blob>,
    ) -> StartCommand {
        StartCommand {
            verbose,
            keyring,
            dns_config,
            proxy,
            options,
            module,
            arg,
            blobs,
        }
    }
}
