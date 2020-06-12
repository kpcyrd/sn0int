use crate::errors::*;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::collections::HashMap;
use std::result;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Opt {
    key: String,
    value: String,
}

impl Opt {
    pub fn collect(opts: &[Opt]) -> HashMap<String, String> {
        opts.iter()
            .map(|x| (x.key.to_string(), x.value.to_string()))
            .collect()
    }
}

impl FromStr for Opt {
    type Err = Error;

    fn from_str(x: &str) -> Result<Opt> {
        if let Some(idx) = x.find('=') {
            let (key, value) = x.split_at(idx);
            Ok(Opt {
                key: key.to_string(),
                value: value[1..].to_string(),
            })
        } else {
            bail!("Malformed option")
        }
    }
}

impl ToString for Opt {
    fn to_string(&self) -> String {
        format!("{}={}", self.key, self.value)
    }
}

impl Serialize for Opt {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Opt {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}
