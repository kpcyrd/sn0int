use crate::errors::*;

use std::collections::HashMap;
use crate::hlua::AnyLuaValue;
use serde_json::{self, Deserializer, Value, Number, Map};


pub fn decode<T: AsRef<[u8]>>(x: T) -> Result<AnyLuaValue> {
    let v: Value = serde_json::from_slice(x.as_ref())
                        .context("deserialize failed")?;
    let v: LuaJsonValue = v.into();
    Ok(v.into())
}

pub fn encode(v: AnyLuaValue) -> Result<String> {
    let v: LuaJsonValue = v.into();
    let v: Value = v.into();
    let s = serde_json::to_string(&v)
        .context("serialize failed")?;
    Ok(s)
}

pub fn decode_stream(x: &str) -> Result<Vec<AnyLuaValue>> {
    let stream = Deserializer::from_str(x).into_iter::<Value>();

    let list = stream
        .filter_map(|x| x.ok())
        .map(|x| LuaJsonValue::from(x).into())
        .collect();

    Ok(list)
}

pub fn lua_array_is_list(array: &[(AnyLuaValue, AnyLuaValue)]) -> bool {
    if !array.is_empty() {
        let first = &array[0];
        matches!(first.0, AnyLuaValue::LuaNumber(_))
    } else {
        // true // TODO: this breaks unserialize
        false
    }
}

#[derive(Debug)]
pub enum LuaJsonValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<LuaJsonValue>),
    Object(HashMap<String, LuaJsonValue>),
}

impl From<LuaJsonValue> for AnyLuaValue {
    fn from(value: LuaJsonValue) -> AnyLuaValue {
        match value {
            LuaJsonValue::Null => AnyLuaValue::LuaNil,
            LuaJsonValue::Bool(v) => AnyLuaValue::LuaBoolean(v),
            // TODO: not sure if this might fail
            LuaJsonValue::Number(v) => AnyLuaValue::LuaNumber(v.as_f64().unwrap()),
            LuaJsonValue::String(v) => AnyLuaValue::LuaString(v),
            LuaJsonValue::Array(v) => AnyLuaValue::LuaArray(v.into_iter().enumerate()
                .map(|(i, x)| (AnyLuaValue::LuaNumber((i+1) as f64), x.into()))
                .collect()
            ),
            LuaJsonValue::Object(v) => AnyLuaValue::LuaArray(v.into_iter()
                .map(|(k, v)| (AnyLuaValue::LuaString(k), v.into()))
                .collect()
            ),
        }
    }
}

impl From<AnyLuaValue> for LuaJsonValue {
    fn from(x: AnyLuaValue) -> LuaJsonValue {
        match x {
            AnyLuaValue::LuaNil => LuaJsonValue::Null,
            AnyLuaValue::LuaBoolean(v) => LuaJsonValue::Bool(v),
            AnyLuaValue::LuaString(v) => LuaJsonValue::String(v),
            AnyLuaValue::LuaAnyString(v) => LuaJsonValue::Array(v.0.into_iter()
                .map(|x| LuaJsonValue::Number(x.into()))
                .collect()
            ),
            AnyLuaValue::LuaNumber(v) => {
                // this is needed or every number is detected as float
                LuaJsonValue::Number(if v % 1f64 == 0f64 {
                    (v as u64).into()
                } else {
                    Number::from_f64(v).expect("invalid LuaJson::Number")
                })
            },
            AnyLuaValue::LuaArray(v) => {
                if lua_array_is_list(&v) {
                    LuaJsonValue::Array(v.into_iter()
                        .map(|(_, v)| v.into())
                        .collect()
                    )
                } else {
                    LuaJsonValue::Object(v.into_iter()
                        .filter_map(|(k, v)| match k {
                            AnyLuaValue::LuaString(k) => Some((k, v.into())),
                            _ => None,
                        })
                        .collect()
                    )
                }
            },
            AnyLuaValue::LuaOther => LuaJsonValue::Null,
        }
    }
}

impl From<LuaJsonValue> for serde_json::Value {
    fn from(value: LuaJsonValue) -> serde_json::Value {
        match value {
            LuaJsonValue::Null => Value::Null,
            LuaJsonValue::Bool(v) => Value::Bool(v),
            LuaJsonValue::Number(v) => Value::Number(v),
            LuaJsonValue::String(v) => Value::String(v),
            LuaJsonValue::Array(v) => Value::Array(v.into_iter()
                .map(|x| x.into())
                .collect()
            ),
            LuaJsonValue::Object(v) => Value::Object(v.into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<Map<_, _>>()
            ),
        }
    }
}

impl From<serde_json::Value> for LuaJsonValue {
    fn from(x: serde_json::Value) -> LuaJsonValue {
        match x {
            Value::Null => LuaJsonValue::Null,
            Value::Bool(v) => LuaJsonValue::Bool(v),
            Value::Number(v) => LuaJsonValue::Number(v),
            Value::String(v) => LuaJsonValue::String(v),
            Value::Array(v) => LuaJsonValue::Array(v.into_iter()
                .map(|x| x.into())
                .collect()
            ),
            Value::Object(v) => LuaJsonValue::Object(v.into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<HashMap<_, _>>()
            ),
        }
    }
}
