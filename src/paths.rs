use crate::config::Config;
use crate::errors::*;
use crate::workspaces::Workspace;

use dirs;
use std::fs;
use std::path::PathBuf;


pub fn sn0int_dir() -> Result<PathBuf> {
    let path = dirs::data_dir()
        .ok_or_else(|| format_err!("Failed to find data directory"))?;
    let path = path.join("sn0int");
    fs::create_dir_all(&path)
        .context("Failed to create data directory")?;
    Ok(path)
}

pub fn history_path() -> Result<PathBuf> {
    let path = sn0int_dir()?;
    let path = path.join("history");
    Ok(path)
}

pub fn module_dir() -> Result<PathBuf> {
    let path = sn0int_dir()?;
    let path = path.join("modules");
    fs::create_dir_all(&path)
        .context("Failed to create module directory")?;
    Ok(path)
}

pub fn data_dir() -> Result<PathBuf> {
    let path = sn0int_dir()?
        .join("data");
    fs::create_dir_all(&path)
        .context("Failed to create module directory")?;
    Ok(path)
}

pub fn workspace_dir(workspace: &Workspace) -> Result<PathBuf> {
    let path = sn0int_dir()?
        .join("data")
        .join(workspace.as_str());
    fs::create_dir_all(&path)
        .context("Failed to create module directory")?;
    Ok(path)
}

pub fn blobs_dir(workspace: &Workspace) -> Result<PathBuf> {
    let path = workspace_dir(workspace)?
        .join("blobs");
    fs::create_dir_all(&path)
        .context("Failed to create module directory")?;
    Ok(path)
}

pub fn cache_dir() -> Result<PathBuf> {
    let path = dirs::cache_dir()
        .ok_or_else(|| format_err!("Failed to find cache directory"))?;
    let path = path.join("sn0int");
    fs::create_dir_all(&path)
        .context("Failed to create cache directory")?;
    Ok(path)
}

fn print_path<D: std::fmt::Debug>(k: &str, v: D) {
    println!("{:30}: {:?}", k, v);
}

pub fn run(config: &Config) -> Result<()> {
    print_path("config_file", Config::path()?);
    print_path("data_dir", data_dir()?);
    print_path("module_dir", module_dir()?);

    for (k, v) in &config.namespaces {
        print_path(&format!("modules({})", k), v);
    }

    print_path("cache_dir", cache_dir()?);
    Ok(())
}
