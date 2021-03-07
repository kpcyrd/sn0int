use std::fmt;
use std::result;
use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{self, Visitor};
use serde::de::SeqAccess;
use std::marker::PhantomData;


#[derive(Debug, Serialize, Deserialize)]
pub struct StringOrBytes(#[serde(deserialize_with="string_or_bytes")] pub Vec<u8>);

pub fn string_or_bytes<'de, D>(deserializer: D) -> result::Result<Vec<u8>, D::Error> where D: Deserializer<'de> {
    struct StringOrBytes(PhantomData<fn() -> Vec<u8>>);

    impl<'de> Visitor<'de> for StringOrBytes {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or sequence")
        }

        fn visit_str<E>(self, value: &str) -> result::Result<Vec<u8>, E>
        where
            E: de::Error,
        {
            Ok(value.as_bytes().to_vec())
        }

        fn visit_seq<V>(self, mut visitor: V) -> result::Result<Vec<u8>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut vec = Vec::new();

            while let Some(element) = visitor.next_element()? {
                vec.push(element)
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_any(StringOrBytes(PhantomData))
}

pub fn opt_string_or_bytes<'de, D>(deserializer: D) -> result::Result<Option<Vec<u8>>, D::Error> where D: Deserializer<'de> {
    Option::<StringOrBytes>::deserialize(deserializer)
        .map(|opt_wrapped: Option<StringOrBytes>| {
            opt_wrapped.map(|wrapped: StringOrBytes| wrapped.0)
        })
}
