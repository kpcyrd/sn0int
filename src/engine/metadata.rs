use errors::*;

use std::str::FromStr;


#[derive(Debug, PartialEq)]
pub enum EntryType {
    Description,
    Version,
    Argument,
}

impl FromStr for EntryType {
    type Err = Error;

    fn from_str(s: &str) -> Result<EntryType> {
        match s {
            "Description" => Ok(EntryType::Description),
            "Version" => Ok(EntryType::Version),
            "Argument" => Ok(EntryType::Argument),
            x => bail!("Unknown EntryType: {:?}", x),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Argument {
    Domains,
    Subdomains,
}

impl FromStr for Argument {
    type Err = Error;

    fn from_str(s: &str) -> Result<Argument> {
        match s {
            "domains" => Ok(Argument::Domains),
            "subdomains" => Ok(Argument::Subdomains),
            x => bail!("Unknown Argument: {:?}", x),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Metadata {
    pub description: String,
    pub version: String,
    pub argument: Argument,
}

impl Metadata {
    pub fn parse(code: &str) -> Result<Metadata> {
        let (_, lines) = metalines(code)
            .map_err(|_| format_err!("Failed to parse header"))?;

        let mut data = NewMetadata::default();

        for (k, v) in lines {
            match k {
                EntryType::Description => data.description = Some(v),
                EntryType::Version => data.version = Some(v),
                EntryType::Argument => data.argument = Some(v),
            }
        }

        data.try_from()
    }
}

#[derive(Default)]
pub struct NewMetadata<'a> {
    pub description: Option<&'a str>,
    pub version: Option<&'a str>,
    pub argument: Option<&'a str>,
}

impl<'a> NewMetadata<'a> {
    fn try_from(self) -> Result<Metadata> {
        let description = self.description.ok_or_else(|| format_err!("Description is required"))?;
        let version = self.version.ok_or_else(|| format_err!("Version is required"))?;
        let argument = self.argument.ok_or_else(|| format_err!("Argument is required"))?;

        let argument = argument.parse()?;

        Ok(Metadata {
            description: description.to_string(),
            version: version.to_string(),
            argument,
        })
    }
}

named!(metaline<&str, (EntryType, &str)>, do_parse!(
    tag!("-- ") >>
    name: map_res!(take_until!(": "), EntryType::from_str) >>
    tag!(": ") >>
    value: take_until!("\n") >>
    tag!("\n") >>
    (
        (name, value)
    )
));

named!(metalines<&str, Vec<(EntryType, &str)>>, do_parse!(
    lines: fold_many0!(metaline, Vec::new(), |mut acc: Vec<_>, item| {
        acc.push(item);
        acc
    }) >>
    tag!("\n") >>
    (lines)
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_simple() {
        let metadata = Metadata::parse(r#"-- Description: Hello world, this is my description
-- Version: 1.0.0
-- Argument: domains

"#).expect("parse");
        assert_eq!(metadata, Metadata {
            description: "Hello world, this is my description".to_string(),
            version: "1.0.0".to_string(),
            argument: Argument::Domains,
        });
    }
}
