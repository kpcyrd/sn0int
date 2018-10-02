use errors::*;
use args::{Args, Publish};
use api::{API_URL, Client};
use auth;
use sn0int_common::metadata::Metadata;
use std::fs;
use std::path::Path;
use term;
use worker;


pub fn run_publish(_args: &Args, publish: &Publish) -> Result<()> {
    let session = auth::load_token()
        .context("Failed to load auth token")?;

    let mut client = Client::new(API_URL)?;
    client.authenticate(session);

    let path = Path::new(&publish.path);
    let name = path.file_stem().ok_or(format_err!("Couldn't get file name"))?;
    let ext = path.extension().ok_or(format_err!("Couldn't get file extension"))?;

    if ext != "lua" {
        bail!("File extension has to be .lua");
    }

    let name = name.to_os_string().into_string()
        .map_err(|_| format_err!("Failed to decode file name"))?;

    let code = fs::read_to_string(path)
        .context("Failed to read module")?;
    let metadata = code.parse::<Metadata>()?;

    let label = format!("Uploading {} {} ({:?})", name, metadata.version, path);
    let result = worker::spawn_fn(&label, || {
        client.publish_module(&name, code.to_string())
    }, false)?;

    term::info(&format!("Published as {}/{} {}", result.author,
                                                 result.name,
                                                 result.version));

    Ok(())
}
