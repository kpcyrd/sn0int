use crate::errors::*;
use crate::workspaces::Workspace;

use dirs;
use std::fs;
use std::path::PathBuf;


pub fn data_dir() -> Result<PathBuf> {
    let path = dirs::data_dir()
        .ok_or_else(|| format_err!("Failed to find data directory"))?;
    let path = path.join("sn0int");
    fs::create_dir_all(&path)
        .context("Failed to create data directory")?;
    Ok(path)
}

pub fn history_path() -> Result<PathBuf> {
    let path = data_dir()?;
    let path = path.join("history");
    Ok(path)
}

pub fn module_dir() -> Result<PathBuf> {
    let path = data_dir()?;
    let path = path.join("modules");
    fs::create_dir_all(&path)
        .context("Failed to create module directory")?;
    Ok(path)
}

pub fn blobs_dir(workspace: &Workspace) -> Result<PathBuf> {
    let path = data_dir()?;
    let path = path
        .join("blobs")
        .join(workspace.to_string());
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
