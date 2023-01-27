use crate::errors::*;
use serde::{Serialize, Deserialize};

use crate::engine::Module;
use crate::hlua::AnyLuaValue;
use crate::json::LuaJsonValue;
use crate::paths;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fmt;
use std::str::FromStr;
use std::path::{Path, PathBuf};
use sn0int_common::ModuleID;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyName {
    pub namespace: String,
    pub name: String,
}

impl KeyName {
    pub fn new<I: Into<String>, J: Into<String>>(namespace: I, name: J) -> KeyName {
        KeyName {
            namespace: namespace.into(),
            name: name.into(),
        }
    }

    pub fn for_each(k: &str, v: &HashMap<String, Option<String>>) -> Vec<KeyName> {
        v.iter()
            .map(move |(x, _)| KeyName::new(k, x.as_str()))
            .collect()
    }
}

impl FromStr for KeyName {
    type Err = Error;

    fn from_str(x: &str) -> Result<KeyName> {
        if let Some(idx) = x.find(':') {
            let (namespace, name) = x.split_at(idx);
            let namespace = namespace.to_string();
            let name = name[1..].to_string();

            if namespace.is_empty() {
                bail!("Namespace can not be empty");
            }

            if name.is_empty() {
                bail!("Name can not be empty");
            }

            Ok(KeyName {
                namespace,
                name,
            })
        } else {
            bail!("Missing namespace")
        }
    }
}

impl fmt::Display for KeyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.name)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KeyRing {
    keys: HashMap<String, HashMap<String, Option<String>>>,
    grants: HashMap<String, HashSet<ModuleID>>,
}

impl KeyRing {
    pub fn path() -> Result<PathBuf> {
        let path = paths::sn0int_dir()?;
        let path = path.join("keyring.json");
        Ok(path)
    }

    pub fn init() -> Result<KeyRing> {
        let path = Self::path()?;

        let keyring = if path.exists() {
            Self::load(&path)
                .context("Failed to load keyring")?
        } else {
            KeyRing::default()
        };

        Ok(keyring)
    }

    pub fn load(path: &Path) -> Result<KeyRing> {
        let buf = fs::read(path)
            .context("Failed to read keyring file")?;
        serde_json::from_slice(&buf)
            .map_err(Error::from)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let buf = serde_json::to_string(&self)?;
        fs::write(path, buf)
            .context("Failed to save keyring")?;
        Ok(())
    }

    pub fn insert(&mut self, key: KeyName, secret: Option<String>) -> Result<()> {
        // get the namespace or create a new one
        let mut x = self.keys.remove(&key.namespace)
            .unwrap_or_default();
        // insert key into namespace
        x.insert(key.name, secret);
        // add namespace backinto keyring
        self.keys.insert(key.namespace, x);
        // save keyring
        self.save()
    }

    pub fn delete(&mut self, key: KeyName) -> Result<()> {
        if let Some(mut x) = self.keys.remove(&key.namespace) {
            // remove the key we want to delete
            x.remove(&key.name);

            // if there are still keys left in the namespace
            if !x.is_empty() {
                self.keys.insert(key.namespace, x);
            }

            // save keyring
            self.save()
        } else {
            Ok(())
        }
    }

    pub fn list(&self) -> Vec<KeyName> {
        self.keys.iter()
            .flat_map(|(k, v)| KeyName::for_each(k, v))
            .collect()
    }

    pub fn list_for(&self, namespace: &str) -> Vec<KeyName> {
        self.keys.iter()
            .filter(|(k, _)| k.as_str() == namespace)
            .flat_map(|(k, v)| KeyName::for_each(k, v))
            .collect()
    }

    pub fn get(&self, key: &KeyName) -> Option<KeyRingEntry> {
        let x = self.keys.get(&key.namespace)?;
        let secret_key = x.get(&key.name)?;

        Some(KeyRingEntry {
            namespace: key.namespace.to_owned(),
            access_key: key.name.to_owned(),
            secret_key: secret_key.to_owned(),
        })
    }

    pub fn get_all_for(&self, namespace: &str) -> Vec<KeyRingEntry> {
        self.list_for(namespace)
            .into_iter()
            .flat_map(|x| self.get(&x))
            .collect()
    }

    pub fn unauthorized_namespaces<'a>(&self, module: &'a Module) -> Vec<&'a String> {
        module.keyring_access().iter()
            .filter(|namespace| !self.is_access_granted(module, namespace))
            .collect()
    }

    pub fn grant_access(&mut self, module: &Module, namespace: String) {
        let mut grants = self.grants.remove(&namespace)
            .unwrap_or_default();
        grants.insert(module.id());
        self.grants.insert(namespace, grants);
    }

    pub fn is_access_granted(&self, module: &Module, namespace: &str) -> bool {
        if let Some(grants) = self.grants.get(namespace) {
            grants.contains(&module.id())
        } else {
            false
        }
    }

    pub fn request_keys(&self, module: &Module) -> Vec<KeyRingEntry> {
        // TODO: we probably want to randomize the order
        module.keyring_access().iter()
            .filter(|namespace| self.is_access_granted(module, namespace))
            .flat_map(|namespace| self.list_for(namespace))
            .flat_map(|x| self.get(&x))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyRingEntry {
    pub namespace: String,
    pub access_key: String,
    pub secret_key: Option<String>,
}

impl KeyRingEntry {
    pub fn to_lua(&self) -> Result<AnyLuaValue> {
        let v = serde_json::to_value(self)?;
        let v = LuaJsonValue::from(v).into();
        Ok(v)
    }

    pub fn matches(&self, query: &str) -> bool {
        if let Some(idx) = query.find(':') {
            let (namespace, access_key) = query.split_at(idx);
            let access_key = &access_key[1..];

            self.namespace == namespace && self.access_key == access_key
        } else {
            self.namespace == query
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_keyname() {
        let x = KeyName::from_str("a:b").unwrap();
        assert_eq!(x, KeyName {
            namespace: "a".into(),
            name: "b".into(),
        });
    }

    #[test]
    fn test_invalid_keyname() {
        assert!(KeyName::from_str("a:").is_err());
        assert!(KeyName::from_str(":a").is_err());
        assert!(KeyName::from_str(":").is_err());
        assert!(KeyName::from_str("a").is_err());
        assert!(KeyName::from_str("").is_err());
    }
}
