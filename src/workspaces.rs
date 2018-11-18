use errors::*;

use std::ffi::OsStr;
use std::fs;
use paths;


pub fn list() -> Result<Vec<String>> {
    let mut workspaces = Vec::new();

    for entry in fs::read_dir(paths::data_dir()?)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension() != Some(OsStr::new("db")) {
            continue;
        }

        let name = match path.file_stem() {
            Some(name) => name,
            _ => continue,
        };

        let name = name.to_str()
            .ok_or_else(|| format_err!("Workspace has invalid name: {:?}", name))?;

        workspaces.push(name.to_string());
    }

    Ok(workspaces)
}
