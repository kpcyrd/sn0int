use hlua::{AnyHashableLuaValue, AnyLuaValue};
use std::collections::{self, HashMap};

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

impl Into<HashMap<AnyHashableLuaValue, AnyLuaValue>> for LuaMap {
    fn into(self: LuaMap) -> HashMap<AnyHashableLuaValue, AnyLuaValue> {
        self.0
    }
}

impl Into<AnyLuaValue> for LuaMap {
    fn into(self: LuaMap) -> AnyLuaValue {
        AnyLuaValue::LuaArray(
            self.into_iter()
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
