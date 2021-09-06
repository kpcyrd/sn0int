use crate::errors::*;
use crate::hlua::{AnyHashableLuaValue, AnyLuaValue, AnyLuaString};
use std::collections::{self, HashMap};
use crate::json::LuaJsonValue;

pub fn from_lua<T>(x: LuaJsonValue) -> Result<T>
    where for<'de> T: serde::Deserialize<'de>
{
    serde_json::from_value(x.into())
        .map_err(Error::from)
}

#[derive(Debug, Default)]
pub struct LuaMap(HashMap<AnyHashableLuaValue, AnyLuaValue>);

impl LuaMap {
    #[inline]
    pub fn new() -> LuaMap {
        LuaMap::default()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn insert<K: Into<String>, V: Into<AnyLuaValue>>(&mut self, k: K, v: V) {
        self.0.insert(AnyHashableLuaValue::LuaString(k.into()), v.into());
    }

    #[inline]
    pub fn insert_str<K: Into<String>, V: Into<String>>(&mut self, k: K, v: V) {
        self.0.insert(AnyHashableLuaValue::LuaString(k.into()), AnyLuaValue::LuaString(v.into()));
    }

    #[inline]
    pub fn insert_num<K: Into<String>>(&mut self, k: K, v: f64) {
        self.0.insert(AnyHashableLuaValue::LuaString(k.into()), AnyLuaValue::LuaNumber(v));
    }

    pub fn insert_serde<K: Into<String>, S: serde::Serialize>(&mut self, k: K, v: S) -> Result<()> {
        let v = serde_json::to_value(v)?;
        self.0.insert(AnyHashableLuaValue::LuaString(k.into()), LuaJsonValue::from(v).into());
        Ok(())
    }
}

impl IntoIterator for LuaMap {
    type Item = (AnyHashableLuaValue, AnyLuaValue);
    type IntoIter = collections::hash_map::IntoIter<AnyHashableLuaValue, AnyLuaValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<HashMap<String, String>> for LuaMap {
    fn from(x: HashMap<String, String>) -> LuaMap {
        let mut map = LuaMap::new();
        for (k, v) in x {
            map.insert_str(k, v);
        }
        map
    }
}

impl From<HashMap<AnyHashableLuaValue, AnyLuaValue>> for LuaMap {
    fn from(x: HashMap<AnyHashableLuaValue, AnyLuaValue>) -> LuaMap {
        LuaMap(x)
    }
}

impl From<Vec<(AnyLuaValue, AnyLuaValue)>> for LuaMap {
    fn from(x: Vec<(AnyLuaValue, AnyLuaValue)>) -> LuaMap {
        let mut map = LuaMap::new();

        for (k, v) in x {
            // TODO: handle unknown types
            if let AnyLuaValue::LuaString(k) = k {
                map.insert(k, v);
            }
        }

        map
    }
}

impl From<LuaMap> for HashMap<AnyHashableLuaValue, AnyLuaValue> {
    fn from(map: LuaMap) -> Self {
        map.0
    }
}

impl From<LuaMap> for AnyLuaValue {
    fn from(map: LuaMap) -> AnyLuaValue {
        AnyLuaValue::LuaArray(
            map.into_iter()
                .filter_map(|(k, v)| {
                    match k {
                        AnyHashableLuaValue::LuaString(x) => Some((AnyLuaValue::LuaString(x), v)),
                        _ => None, // TODO: unknown types are discarded
                    }
                })
                .collect()
        )
    }
}

#[derive(Debug, Default)]
pub struct LuaList(Vec<(AnyLuaValue, AnyLuaValue)>);

impl LuaList {
    #[inline]
    pub fn new() -> LuaList {
        LuaList::default()
    }

    pub fn push<V: Into<AnyLuaValue>>(&mut self, v: V) {
        let idx = self.0.len() + 1;
        self.0.push((AnyLuaValue::LuaNumber(idx as f64), v.into()));
    }

    pub fn push_str<I: Into<String>>(&mut self, v: I) {
        self.push(AnyLuaValue::LuaString(v.into()))
    }

    pub fn push_serde<S: serde::Serialize>(&mut self, v: S) -> Result<()> {
        let v = serde_json::to_value(v)?;
        self.push(LuaJsonValue::from(v));
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<LuaList> for AnyLuaValue {
    fn from(list: LuaList) -> AnyLuaValue {
        AnyLuaValue::LuaArray(list.0)
    }
}

pub fn byte_array(bytes: AnyLuaValue) -> Result<Vec<u8>> {
    match bytes {
        AnyLuaValue::LuaAnyString(bytes) => Ok(bytes.0),
        AnyLuaValue::LuaString(bytes) => Ok(bytes.into_bytes()),
        AnyLuaValue::LuaArray(bytes) => {
            Ok(bytes.into_iter()
                .map(|num| match num.1 {
                    AnyLuaValue::LuaNumber(num) if (0.0..=255.0).contains(&num) && (num % 1.0 == 0.0) =>
                            Ok(num as u8),
                    AnyLuaValue::LuaNumber(num) =>
                            Err(format_err!("number is out of range: {:?}", num)),
                    _ => Err(format_err!("unexpected type: {:?}", num)),
                })
                .collect::<Result<_>>()?)
        },
        _ => Err(format_err!("invalid type: {:?}", bytes)),
    }
}

pub fn lua_bytes(bytes: &[u8]) -> AnyLuaValue {
    let bytes = AnyLuaString(bytes.to_vec());
    AnyLuaValue::LuaAnyString(bytes)
}
