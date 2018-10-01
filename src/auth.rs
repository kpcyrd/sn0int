use errors::*;
use opener;
use std::thread;
use std::time::Duration;
use api::Client;
use sn0int_common::WhoamiResponse;
use term;

const LOGIN_URL: &str = "http://[::1]:8000/api/v0/login";
const WHOAMI_URL: &str = "http://[::1]:8000/api/v0/whoami";


pub fn run_login() -> Result<()> {
    let session = Client::random_session();
    let url = format!("{}/{}", LOGIN_URL, session);

    // TODO: check if already logged in

    term::success(&format!("Opening url: {}", url));
    opener::open(url)?;

    let client = Client::new()?;
    let url = format!("{}/{}", WHOAMI_URL, session);

    for _ in 0..24 {
        thread::sleep(Duration::from_secs(5));

        let resp = client.get::<WhoamiResponse>(&url)?;
        if let Some(user) = resp.user {
            term::info(&format!("Logged in as {:?}", user));
            return Ok(());
        }
    }

    bail!("Authentication timed out")
}
