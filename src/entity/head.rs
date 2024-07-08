use std::{path::PathBuf, str::FromStr};

use crate::consts::HEAD_PATH;

use super::{object::hash::Hash, refs::Ref};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub enum Head {
    Detached(Hash),
    Ref(Ref),
}
impl Head {
    fn get_path() -> PathBuf {
        PathBuf::from(HEAD_PATH)
    }

    pub fn read() -> Result<Self> {
        let content = std::fs::read_to_string(Self::get_path())?;
        content.trim().parse()
    }

    pub fn write(&self) -> Result<()> {
        std::fs::write(Self::get_path(), self.to_string())?;
        Ok(())
    }
}
impl FromStr for Head {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.starts_with("ref:") {
            let ref_name = s
                .strip_prefix("ref: ")
                .ok_or(anyhow!("Invalid ref: {}", s))?;
            Ok(Head::Ref(ref_name.parse()?))
        } else {
            Ok(Head::Detached(s.parse()?))
        }
    }
}
impl std::fmt::Display for Head {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Head::Detached(hash) => write!(f, "{}", hash),
            Head::Ref(r) => write!(f, "ref: {}", r),
        }
    }
}
