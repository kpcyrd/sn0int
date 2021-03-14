use crate::errors::*;
use crate::paths;
use crate::worker::{EventWithCallback, Event2};
use crate::workspaces::Workspace;

use bytes::Bytes;
use std::fs;
use std::path::{Path, PathBuf};
use std::result;
use std::sync::mpsc;

pub use sn0int_std::blobs::Blob;

impl EventWithCallback for Blob {
    type Payload = ();

    #[inline(always)]
    fn with_callback(self, tx: mpsc::Sender<result::Result<Self::Payload, String>>) -> Event2 {
        Event2::Blob((self, tx))
    }
}

pub struct BlobStorage {
    path: PathBuf,
}

impl BlobStorage {
    #[inline]
    pub fn new<I: Into<PathBuf>>(path: I) -> BlobStorage {
        BlobStorage {
            path: path.into(),
        }
    }

    #[inline]
    pub fn workspace(workspace: &Workspace) -> Result<BlobStorage> {
        let path = paths::blobs_dir(workspace)?;
        Ok(BlobStorage::new(path))
    }

    #[inline]
    pub fn join(&self, id: &str) -> Result<PathBuf> {
        if !id.chars().all(char::is_alphanumeric) {
            bail!("blob id contains invalid characters");
        }
        Ok(self.path.join(id))
    }

    #[inline(always)]
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn load(&self, id: &str) -> Result<Blob> {
        let path = self.join(id)?;

        debug!("Loading blob from {:?}", path);
        let bytes = fs::read(path)
            .context("Failed to read blob")?;

        Ok(Blob {
            id: id.to_string(),
            bytes: Bytes::from(bytes),
        })
    }

    pub fn save(&self, blob: &Blob) -> Result<()> {
        let path = self.join(&blob.id)?;

        debug!("Writing blob to {:?}", path);
        fs::write(path, &blob.bytes)
            .context("Failed to write blob")?;

        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let path = self.join(id)?;
        debug!("Deleting blob: {:?}", path);
        fs::remove_file(path)
            .context("Failed to delete blob")?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<String>> {
        let mut blobs = Vec::new();
        for entry in fs::read_dir(&self.path)? {
            let blob = entry?
                .file_name()
                .into_string()
                .map_err(|_| format_err!("Invalid filename"))?;
            blobs.push(blob);
        }
        Ok(blobs)
    }

    pub fn stat(&self, id: &str) -> Result<u64> {
        let path = self.join(id)?;
        debug!("Stat-ing blob: {:?}", path);
        let md = fs::metadata(path)
            .context("Failed to stat blob")?;
        Ok(md.len())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[inline]
    fn blob() -> (Bytes, Blob) {
        let bytes = Bytes::from(&b"asdf"[..]);
        (bytes.clone(), Blob::create(bytes))
    }

    #[test]
    fn test_blobstorage_save() {
        let dir = tempfile::tempdir().expect("tempdir");
        let s = BlobStorage::new(dir.path());

        let (_, blob1) = blob();
        s.save(&blob1).expect("save failed");

        let blob2 = s.load("DTTV3EjpHBNJx3Zw7eJsVPm4bYXKmNkJQpVNkcvTtTSz").expect("load failed");

        assert_eq!(blob1, blob2);
    }

    #[test]
    fn test_blobstorage_load_failure() {
        let dir = tempfile::tempdir().expect("tempdir");
        let s = BlobStorage::new(dir.path());

        let result = s.load("DTTV3EjpHBNJx3Zw7eJsVPm4bYXKmNkJQpVNkcvTtTSz");

        assert!(result.is_err());
    }

    #[test]
    fn test_path_validation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let s = BlobStorage::new(dir.path());

        let result = s.load("../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../etc/passwd");

        assert!(result.is_err());
    }
}
