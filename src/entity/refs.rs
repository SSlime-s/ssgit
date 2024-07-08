use std::{path::PathBuf, str::FromStr};

use anyhow::{bail, Result};

use crate::consts::REFS_DIRECTORY;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ref {
    Branch(String),
    Tag(String),
}
impl Ref {
    fn name_to_path(name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    pub fn branch_names() -> Result<Vec<String>> {
        let mut ret = Vec::new();

        let path = PathBuf::from(REFS_DIRECTORY).join("heads");
        if !path.exists() {
            return Ok(ret);
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            ret.push(name);
        }

        Ok(ret)
    }

    pub fn to_path(&self) -> PathBuf {
        let mut ret = PathBuf::from(REFS_DIRECTORY);

        match self {
            Ref::Branch(name) => {
                ret.push("heads");
                ret.push(Self::name_to_path(name));
            }
            Ref::Tag(name) => {
                ret.push("tags");
                ret.push(Self::name_to_path(name));
            }
        }

        ret
    }

    pub fn write_hash(&self, hash: &str) -> Result<()> {
        let path = self.to_path();
        if path.is_dir() {
            bail!("Ref is a directory: {}", self.to_string());
        }
        if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap())?;
        }

        std::fs::write(path, hash)?;

        Ok(())
    }

    pub fn read_hash(&self) -> Result<Option<String>> {
        let path = self.to_path();

        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        Ok(Some(content.trim().to_string()))
    }
}
impl FromStr for Ref {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.starts_with("refs/heads/") {
            Ok(Ref::Branch(s.replace("refs/heads/", "")))
        } else if s.starts_with("refs/tags/") {
            Ok(Ref::Tag(s.replace("refs/tags/", "")))
        } else {
            bail!("Invalid ref: {}", s);
        }
    }
}
impl std::fmt::Display for Ref {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Ref::Branch(name) => write!(f, "refs/heads/{}", name),
            Ref::Tag(name) => write!(f, "refs/tags/{}", name),
        }
    }
}
