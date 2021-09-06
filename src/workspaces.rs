use crate::errors::*;

use crate::blobs::BlobStorage;
use crate::paths;
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;


#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Workspace {
    s: String,
}

impl Workspace {
    #[inline]
    pub fn db_path(&self) -> Result<PathBuf> {
        Ok(paths::workspace_dir(self)?
            .join("db.sqlite"))
    }

    #[inline]
    pub fn usage_human(&self) -> Result<String> {
        let usage = self.usage()?;
        Ok(bytesize::to_string(usage, false))
    }

    pub fn usage(&self) -> Result<u64> {
        let blobs = BlobStorage::workspace(self)?;

        let mut sum = fs::metadata(self.db_path()?)?.len();
        for entry in fs::read_dir(blobs.path())? {
            sum += fs::metadata(entry?.path())?.len();
        }

        Ok(sum)
    }

    pub fn delete(&self) -> Result<()> {
        self.migrate()?;

        let path = paths::workspace_dir(self)?;
        fs::remove_dir_all(path)?;

        Ok(())
    }

    pub fn migrate(&self) -> Result<()> {
        // relocate old paths
        let old_db = paths::sn0int_dir()?
            .join(self.to_string() + ".db");
        if old_db.exists() {
            let new_db = paths::workspace_dir(self)?
                .join("db.sqlite");

            fs::rename(old_db, new_db)
                .context("Failed to migrate old db to new location")?;
        }

        let old_blobs_parent = paths::sn0int_dir()?
            .join("blobs");
        let old_blobs = old_blobs_parent
            .join(self.as_str());
        if old_blobs.exists() {
            let new_blobs = paths::workspace_dir(self)?
                .join("blobs");
            fs::rename(old_blobs, new_blobs)
                .context("Failed to migrate old blob folder to new location")?;
        }

        // cleanup
        fs::remove_dir(old_blobs_parent).ok();

        Ok(())
    }
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
    let mut workspaces = HashSet::new();

    for entry in fs::read_dir(paths::data_dir()?)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = path.file_name()
            .ok_or_else(|| format_err!("read_dir returned file with no name"))?;
        let name = name.to_str()
            .ok_or_else(|| format_err!("Workspace has invalid name: {:?}", name))?;

        if let Ok(workspace) = Workspace::from_str(name) {
            workspaces.insert(workspace);
        }
    }

    for entry in fs::read_dir(paths::sn0int_dir()?)? {
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
            workspaces.insert(workspace);
        }
    }

    let mut workspaces = workspaces.into_iter().collect::<Vec<_>>();
    workspaces.sort();
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
