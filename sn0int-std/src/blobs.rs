use blake2::Blake2bVar;
use bytes::Bytes;
use data_encoding::BASE64;
use digest::{Update, VariableOutput};
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::result;

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
        let mut h = Blake2bVar::new(32).unwrap();
        h.update(bytes);
        let output = h.finalize_boxed();
        Self::encode_hash(&output)
    }

    #[inline]
    fn encode_hash(bytes: &[u8]) -> String {
        let x = bs58::encode(bytes).into_string();
        format!("{:0<44}", x)
    }
}

impl Serialize for Blob {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = BASE64.encode(&self.bytes);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Blob {
    #[inline]
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = BASE64.decode(s.as_bytes())
            .map_err(de::Error::custom)?;
        Ok(Blob::create(Bytes::from(bytes)))
    }
}

pub trait BlobState {
    fn register_blob(&self, blob: Blob) -> String;
}


#[cfg(test)]
mod tests {
    use super::*;
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
            id: String::from("DTTV3EjpHBNJx3Zw7eJsVPm4bYXKmNkJQpVNkcvTtTSz"),
            bytes,
        });
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

    #[test]
    fn test_hash_encoding() {
        let x = bs58::decode("22es54J4FbFtpb5D1MtBazVuum4TcqCQ7M9JkmYdmJ8W")
            .into_vec()
            .unwrap();
        let x = Blob::encode_hash(&x);
        assert_eq!(x.len(), 44);
        assert_eq!(x, "22es54J4FbFtpb5D1MtBazVuum4TcqCQ7M9JkmYdmJ8W");
    }

    #[test]
    fn test_hash_encoding_padding() {
        let x = bs58::decode("r6edvU326yvpXLubYacXXSxf2HzqCgzqHUQvpWyNwei")
            .into_vec()
            .unwrap();
        let x = Blob::encode_hash(&x);
        assert_eq!(x.len(), 44);
        assert_eq!(x, "r6edvU326yvpXLubYacXXSxf2HzqCgzqHUQvpWyNwei0");
    }
}
