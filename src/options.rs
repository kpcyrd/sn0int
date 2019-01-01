use crate::errors::*;
use std::collections::HashMap;
use std::str::FromStr;


#[derive(Debug)]
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
