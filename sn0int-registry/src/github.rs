use crate::errors::*;
use std::env;
use reqwest;


pub struct GithubAuthenticator {
    client_id: String,
    client_secret: String,
}

impl GithubAuthenticator {
    pub fn new(client_id: String, client_secret: String) -> GithubAuthenticator {
        GithubAuthenticator {
            client_id,
            client_secret,
        }
    }

    pub fn from_env() -> Result<GithubAuthenticator> {
        let client_id = env::var("GITHUB_CLIENT_ID")
            .context("GITHUB_CLIENT_ID is not set")?;
        let client_secret = env::var("GITHUB_CLIENT_SECRET")
            .context("GITHUB_CLIENT_SECRET is not set")?;
        Ok(GithubAuthenticator::new(client_id, client_secret))
    }

    pub fn get_username(&self, oauth_token: &str) -> Result<String> {
        let url = format!("https://api.github.com/applications/{}/tokens/{}", self.client_id, oauth_token);
        let client = reqwest::Client::new();
        let mut resp = client.get(&url)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .send()?;

        if !resp.status().is_success() {
            bail!("Github returned: {}", resp.status())
        }

        let data = resp.json::<GithubReply>()
            .context("Failed to deserialize github reply")?;

        Ok(data.user.login)
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubReply {
    user: GithubUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    login: String,
    #[serde(rename="type")]
    user_type: String,
}
