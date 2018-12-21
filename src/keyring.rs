use crate::errors::*;

use crate::engine::Module;
use crate::paths;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use std::path::PathBuf;


#[derive(Debug, PartialEq)]
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

pub struct KeyRing {
    path: PathBuf,
    keys: HashMap<String, HashMap<String, Option<String>>>,
}

impl KeyRing {
    pub fn path() -> Result<PathBuf> {
        let path = paths::data_dir()?;
        let path = path.join("keyring.json");
        Ok(path)
    }

    pub fn init() -> Result<KeyRing> {
        let path = Self::path()?;

        if path.exists() {
            Self::load(path)
        } else {
            Ok(KeyRing {
                path,
                keys: HashMap::new(),
            })
        }
    }

    pub fn load(path: PathBuf) -> Result<KeyRing> {
        let buf = fs::read(&path)?;
        let keys = serde_json::from_slice(&buf)?;
        Ok(KeyRing {
            path,
            keys,
        })
    }

    pub fn save(&self) -> Result<()> {
        let buf = serde_json::to_string(&self.keys)?;
        fs::write(&self.path, buf)?;
        Ok(())
    }

    pub fn insert(&mut self, key: KeyName, secret: Option<String>) -> Result<()> {
        // get the namespace or create a new one
        let mut x = self.keys.remove(&key.namespace)
            .unwrap_or_else(|| HashMap::new());
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

    pub fn request_keys(&self, module: &Module) -> Vec<KeyRingEntry> {
        // TODO: we probably want to randomize the order
        module.keyring_access().iter()
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
