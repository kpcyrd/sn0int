use crate::errors::*;

use std::ffi::OsStr;
use std::fs;
use std::str::FromStr;
use crate::paths;


#[derive(Debug, Clone)]
pub struct Workspace {
    s: String,
}

impl FromStr for Workspace {
    type Err = Error;

    fn from_str(s: &str) -> Result<Workspace> {
        if s.is_empty() {
            bail!("Workspace can't be empty")
        }

        if !s.chars().all(char::is_alphanumeric) {
            bail!("Workspace contains invalid characters")
        }

        Ok(Workspace {
            s: s.into(),
        })
    }
}

use std::ops::Deref;
impl Deref for Workspace {
    type Target = String;

    fn deref(&self) -> &String {
        &self.s
    }
}

pub fn list() -> Result<Vec<Workspace>> {
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

        if let Ok(workspace) = Workspace::from_str(name) {
            workspaces.push(workspace);
        }
    }

    Ok(workspaces)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_workspace() {
        let x = Workspace::from_str("abc");
        assert!(x.is_ok());
    }

    #[test]
    fn test_invalid_workspace() {
        let x = Workspace::from_str("/");
        assert!(x.is_err());

        let x = Workspace::from_str(".");
        assert!(x.is_err());

        let x = Workspace::from_str(" ");
        assert!(x.is_err());

        let x = Workspace::from_str("");
        assert!(x.is_err());
    }
}
