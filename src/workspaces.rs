use crate::errors::*;

use crate::paths;
use regex::Regex;
use std::ffi::OsStr;
use std::fs;
use std::str::FromStr;


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

        lazy_static! {
            static ref RE: Regex = Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9\._\-]*[a-zA-Z0-9])?$").unwrap();
        }
        if !RE.is_match(s) {
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

        let x = Workspace::from_str("abc/d");
        assert!(x.is_err());

        let x = Workspace::from_str(".");
        assert!(x.is_err());

        let x = Workspace::from_str("-");
        assert!(x.is_err());

        let x = Workspace::from_str(" ");
        assert!(x.is_err());

        let x = Workspace::from_str("");
        assert!(x.is_err());
    }

    #[test]
    fn test_valid_singlechar() {
        let x = Workspace::from_str("a");
        assert!(x.is_ok());
    }

    #[test]
    fn test_valid_middle_chars() {
        let x = Workspace::from_str("a-b");
        assert!(x.is_ok());

        let x = Workspace::from_str("a_b");
        assert!(x.is_ok());

        let x = Workspace::from_str("example.com");
        assert!(x.is_ok());
    }

    #[test]
    fn test_invalid_middle_chars_at_edge() {
        let x = Workspace::from_str("a-");
        assert!(x.is_err());

        let x = Workspace::from_str("-b");
        assert!(x.is_err());

        let x = Workspace::from_str("-");
        assert!(x.is_err());
    }
}
