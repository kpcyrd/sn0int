use errors::*;
use opener;
use std::fs;
use std::thread;
use std::time::Duration;
use api::Client;
use paths;
use sn0int_common::WhoamiResponse;
use term;

const API_URL: &str = "http://[::1]:8000";

pub fn load_token() -> Result<String> {
    let path = paths::data_dir()?;
    let path = path.join("auth");
    let session = fs::read_to_string(path)?;
    Ok(session.trim().to_string())
}

pub fn save_token(session: &str) -> Result<()> {
    let path = paths::data_dir()?;
    let path = path.join("auth");
    fs::write(path, format!("{}\n", session))?;
    Ok(())
}

fn verify_session(client: &Client, session: &str) -> Result<String> {
    let url = format!("{}/api/v0/whoami/{}", API_URL, session);
    let resp = client.get::<WhoamiResponse>(&url)?;
    if let Some(user) = resp.user {
        Ok(user)
    } else {
        bail!("Session is invalid")
    }
}

pub fn run_login() -> Result<()> {
    let session = Client::random_session();
    let url = format!("{}/api/v0/login/{}", API_URL, session);

    let client = Client::new()?;

    if let Ok(session) = load_token() {
        if let Ok(user) = verify_session(&client, &session) {
            term::info(&format!("Logged in as {:?}", user));
            return Ok(());
        }
    }

    term::success(&format!("Opening url: {}", url));
    opener::open(url)?;

    for _ in 0..24 {
        thread::sleep(Duration::from_secs(5));

        if let Ok(user) = verify_session(&client, &session) {
            save_token(&session)?;
            term::info(&format!("Logged in as {:?}", user));
            return Ok(());
        }
    }

    bail!("Authentication timed out")
}
