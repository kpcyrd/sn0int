use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum Stealth {
    Loud,
    Normal,
    Passive,
    Offline,
}

impl Stealth {
    #[inline]
    pub fn variants() -> &'static [&'static str] {
        &[
            "loud",
            "normal",
            "passive",
            "offline",
        ]
    }

    #[inline(always)]
    fn as_num(&self) -> u8 {
        match self {
            Stealth::Loud => 3,
            Stealth::Normal => 2,
            Stealth::Passive => 1,
            Stealth::Offline => 0,
        }
    }

    #[inline]
    pub fn equal_or_better(&self, other: &Self) -> bool {
        self.as_num() <= other.as_num()
    }
}

impl FromStr for Stealth {
    type Err = Error;

    fn from_str(s: &str) -> Result<Stealth> {
        match s {
            "loud" => Ok(Stealth::Loud),
            // This is also the default level if none is provided
            "normal" => Ok(Stealth::Normal),
            "passive" => Ok(Stealth::Passive),
            "offline" => Ok(Stealth::Offline),
            x => bail!("Unknown stealth: {:?}", x),
        }
    }
}
