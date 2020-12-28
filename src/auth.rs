use crate::errors::*;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use crate::api::Client;
use crate::config::Config;
use crate::paths;
use crate::term;


fn path() -> Result<PathBuf> {
    let path = paths::sn0int_dir()?;
    Ok(path.join("auth"))
}

pub fn load_token() -> Result<String> {
    let session = fs::read_to_string(path()?)?;
    Ok(session.trim().to_string())
}

pub fn save_token(session: &str) -> Result<()> {
    fs::write(path()?, format!("{}\n", session))?;
    Ok(())
}

pub fn run_login(config: &Config) -> Result<()> {
    let mut client = Client::new(config)?;

    if let Ok(session) = load_token() {
        client.authenticate(session);
        if let Ok(user) = client.verify_session() {
            term::info(&format!("Logged in as {:?}", user));
            return Ok(());
        }
    }

    let session = Client::random_session();
    client.authenticate(session.clone());
    let url = format!("{}/auth/{}", config.core.registry, session);

    term::success(&format!("Opening url: {}", url));
    opener::open(url)?;

    for _ in 0..24 {
        thread::sleep(Duration::from_secs(5));

        if let Ok(user) = client.verify_session() {
            save_token(&session)?;
            term::info(&format!("Logged in as {:?}", user));
            return Ok(());
        }
    }

    bail!("Authentication timed out")
}
