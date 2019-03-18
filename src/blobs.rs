use crate::errors::*;
use crate::paths;
use crate::worker::{EventWithCallback, Event2};
use crate::workspaces::Workspace;

use blake2::VarBlake2b;
use digest::{Input, VariableOutput};
use bytes::Bytes;
use serde::ser::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer};
use std::fs;
use std::path::PathBuf;
use std::result;
use std::sync::mpsc;


#[derive(Debug, Clone, PartialEq)]
pub struct Blob {
    pub id: String,
    pub bytes: Bytes,
}

impl Blob {
    pub fn create(bytes: Bytes) -> Blob {
        let id = Self::hash(&bytes);
        Blob {
            id,
            bytes,
        }
    }

    pub fn hash(bytes: &[u8]) -> String {
        let mut h = VarBlake2b::new(16).unwrap();
        h.input(bytes);
        hex::encode(h.vec_result())
    }
}

impl EventWithCallback for Blob {
    type Payload = ();

    fn with_callback(self, tx: mpsc::Sender<result::Result<Self::Payload, String>>) -> Event2 {
        Event2::Blob((self, tx))
    }
}

impl Serialize for Blob {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = base64::encode(&self.bytes);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Blob {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = base64::decode(&s)
            .map_err(de::Error::custom)?;
        Ok(Blob::create(Bytes::from(bytes)))
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

    pub fn workspace(workspace: &Workspace) -> Result<BlobStorage> {
        let path = paths::blobs_dir(workspace)?;
        Ok(BlobStorage::new(path))
    }

    fn join(&self, id: &str) -> Result<PathBuf> {
        if !id.chars().all(char::is_alphanumeric) {
            bail!("blob id contains invalid characters");
        }
        Ok(self.path.join(id))
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
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;
    use serde_json;

    #[inline]
    fn blob() -> (Bytes, Blob) {
        let bytes = Bytes::from(&b"asdf"[..]);
        (bytes.clone(), Blob::create(bytes))
    }

    #[test]
    fn verify_create_blob() {
        let (bytes, blob) = blob();
        assert_eq!(blob, Blob {
            id: String::from("3430b567c4ca76dc5381ae8d07cae024"),
            bytes,
        });
    }

    #[test]
    fn test_blobstorage_save() {
        let dir = tempfile::tempdir().expect("tempdir");
        let s = BlobStorage::new(dir.path());

        let (_, blob1) = blob();
        s.save(&blob1).expect("save failed");

        let blob2 = s.load("3430b567c4ca76dc5381ae8d07cae024").expect("load failed");

        assert_eq!(blob1, blob2);
    }

    #[test]
    fn test_blobstorage_load_failure() {
        let dir = tempfile::tempdir().expect("tempdir");
        let s = BlobStorage::new(dir.path());

        let result = s.load("3430b567c4ca76dc5381ae8d07cae024");

        assert!(result.is_err());
    }

    #[test]
    fn test_path_validation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let s = BlobStorage::new(dir.path());

        let result = s.load("../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../etc/passwd");

        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let (_, blob) = blob();
        let json = serde_json::to_string(&blob).unwrap();
        assert_eq!(&json, "\"YXNkZg==\"");
    }

    #[test]
    fn test_deserialize() {
        let (_, blob1) = blob();
        let blob2: Blob = serde_json::from_str("\"YXNkZg==\"").unwrap();
        assert_eq!(blob1, blob2);
    }
}
