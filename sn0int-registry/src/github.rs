use serde::Deserialize;
use sn0int_registry::errors::*;

pub fn get_username(oauth_token: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let mut resp = client.get("https://api.github.com/user")
        .header("Authorization", format!("token {}", oauth_token))
        .header("User-Agent", "sn0int-registry")
        .send()
        .context("Failed to check access_token")?
        .error_for_status()
        .context("Github returned http error")?;

    let data = resp.json::<GithubUser>()
        .context("Failed to deserialize github reply")?;

    Ok(data.login)
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    login: String,
    #[serde(rename="type")]
    user_type: String,
}
