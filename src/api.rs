use errors::*;
use std::fmt;
use chrootable_https;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use serde::de::DeserializeOwned;
use serde_json;
use sn0int_common::ApiResponse;


pub struct Client {
    client: chrootable_https::Client<chrootable_https::Resolver>,
}

impl Client {
    pub fn new() -> Result<Client> {
        let client = chrootable_https::Client::with_system_resolver()?;
        Ok(Client {
            client,
        })
    }

    pub fn random_session() -> String {
        thread_rng().sample_iter(&Alphanumeric).take(32).collect()
    }

    pub fn get<T: DeserializeOwned + fmt::Debug>(&self, url: &str) -> Result<T> {
        info!("requesting: {:?}", url);
        let resp = self.client.get(&url)?;
        info!("response: {:?}", resp);

        let reply = serde_json::from_slice::<ApiResponse<T>>(&resp.body)?;
        info!("api: {:?}", reply);
        let reply = reply.success()?;
        info!("api(success): {:?}", reply);

        Ok(reply)
    }
}
