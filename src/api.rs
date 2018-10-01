use errors::*;
use std::fmt;
use chrootable_https::{self, HttpClient};
use http::{Request, Uri};
use hyper::Body;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use serde::de::DeserializeOwned;
use serde_json;
use sn0int_common::{ApiResponse, WhoamiResponse};


pub struct Client {
    server: String,
    client: chrootable_https::Client<chrootable_https::Resolver>,
    session: Option<String>,
}

impl Client {
    pub fn new<I: Into<String>>(server: I) -> Result<Client> {
        let client = chrootable_https::Client::with_system_resolver()?;
        Ok(Client {
            server: server.into(),
            client,
            session: None,
        })
    }

    pub fn authenticate<I: Into<String>>(&mut self, session: I) {
        self.session = Some(session.into());
    }

    pub fn random_session() -> String {
        thread_rng().sample_iter(&Alphanumeric).take(32).collect()
    }

    pub fn get<T: DeserializeOwned + fmt::Debug>(&self, url: &str) -> Result<T> {
        let url = url.parse::<Uri>()?;

        info!("requesting: {:?}", url);
        let mut request = Request::builder();

        if let Some(session) = &self.session {
            info!("Adding session token");
            request.header("Auth", session.as_str());
        }

        let request = request.uri(url)
               .body(Body::empty())?;

        let resp = self.client.request(request)?;
        info!("response: {:?}", resp);

        let reply = serde_json::from_slice::<ApiResponse<T>>(&resp.body)?;
        info!("api: {:?}", reply);
        let reply = reply.success()?;
        info!("api(success): {:?}", reply);

        Ok(reply)
    }

    pub fn verify_session(&self) -> Result<String> {
        let url = format!("{}/api/v0/whoami", self.server);
        let resp = self.get::<WhoamiResponse>(&url)?;
        Ok(resp.user)
    }
}
