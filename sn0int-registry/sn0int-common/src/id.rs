use crate::errors::*;
use nom;
use nom::types::CompleteStr;
use serde::{de, Serialize, Serializer, Deserialize, Deserializer};
use std::fmt;
use std::result;
use std::str::FromStr;


fn valid_char(c: char) -> bool {
    nom::is_alphanumeric(c as u8) || c == '-'
}

pub fn valid_name(name: &str) -> Result<()> {
    if token(CompleteStr(name)).is_ok() {
        Ok(())
    } else {
        bail!("String contains invalid character")
    }
}

named!(module<CompleteStr, ModuleID>, do_parse!(
    author: token   >>
    tag!("/")       >>
    name: token     >>
    eof!()          >>
    (
        ModuleID {
            author: author.to_string(),
            name: name.to_string(),
        }
    )
));

named!(token<CompleteStr, CompleteStr>, take_while1!(valid_char));

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ModuleID {
    pub author: String,
    pub name: String,
}

impl fmt::Display for ModuleID {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{}/{}", self.author, self.name)
    }
}

impl FromStr for ModuleID {
    type Err = Error;

    fn from_str(s: &str) -> Result<ModuleID> {
        let (_, module) = module(CompleteStr(s))
            .map_err(|err| format_err!("Failed to parse module id: {:?}", err))?;
        Ok(module)
    }
}

impl Serialize for ModuleID {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ModuleID {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_valid() {
        let result = ModuleID::from_str("kpcyrd/foo").expect("parse");
        assert_eq!(result, ModuleID {
            author: "kpcyrd".to_string(),
            name: "foo".to_string(),
        });
    }

    #[test]
    fn verify_trailing_slash() {
        let result = ModuleID::from_str("kpcyrd/foo/");
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn verify_trailing_data() {
        let result = ModuleID::from_str("kpcyrd/foo/x");
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn verify_empty_author() {
        let result = ModuleID::from_str("/foo");
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn verify_empty_name() {
        let result = ModuleID::from_str("kpcyrd/");
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn verify_missing_slash() {
        let result = ModuleID::from_str("kpcyrdfoo");
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn verify_one_slash() {
        let result = ModuleID::from_str("/");
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn verify_two_slash() {
        let result = ModuleID::from_str("//");
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn verify_empty_str() {
        let result = ModuleID::from_str("");
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn verify_dots() {
        let result = ModuleID::from_str("../..");
        println!("{:?}", result);
        assert!(result.is_err());
    }
}
