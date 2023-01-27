use crate::errors::*;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::result;
use std::str::FromStr;

#[inline(always)]
fn valid_char(c: char) -> bool {
    nom::character::is_alphanumeric(c as u8) || c == '-'
}

pub fn valid_name(name: &str) -> Result<()> {
    if token(name).is_ok() {
        Ok(())
    } else {
        bail!("String contains invalid character")
    }
}

fn module(s: &str) -> nom::IResult<&str, ModuleID> {
    let (input, (author, _, name)) = nom::sequence::tuple((
        token,
        nom::bytes::complete::tag("/"),
        token,
    ))(s)?;
    Ok((input, ModuleID {
        author: author.to_string(),
        name: name.to_string(),
    }))
}

#[inline]
fn token(s: &str) -> nom::IResult<&str, &str> {
    nom::bytes::complete::take_while1(valid_char)(s)
}

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
        let (trailing, module) = module(s)
            .map_err(|err| anyhow!("Failed to parse module id: {:?}", err))?;
        if !trailing.is_empty() {
            bail!("Trailing data in module id");
        }
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
    where
        D: Deserializer<'de>,
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
