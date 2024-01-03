use std::str::FromStr;

use anyhow::bail;

pub mod hash;
pub mod zip;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}
impl FromStr for ObjectType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(Self::Blob),
            "tree" => Ok(Self::Tree),
            "commit" => Ok(Self::Commit),
            _ => bail!("Invalid object type"),
        }
    }
}
impl ToString for ObjectType {
    fn to_string(&self) -> String {
        match self {
            Self::Blob => "blob",
            Self::Tree => "tree",
            Self::Commit => "commit",
        }
        .to_string()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GitObject {
    pub type_: ObjectType,
    pub content: Vec<u8>,
}
impl GitObject {
    pub fn new(type_: ObjectType, content: Vec<u8>) -> Self {
        Self { type_, content }
    }

    pub fn size(&self) -> usize {
        self.content.len()
    }
}
impl TryFrom<&[u8]> for GitObject {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut parts = value.splitn(2, |b| *b == b'\0');

        let header = parts
            .next()
            .ok_or(anyhow::anyhow!("Could not find header"))?;
        let content = parts
            .next()
            .ok_or(anyhow::anyhow!("Could not find content"))?;

        let header = std::str::from_utf8(header)?;
        let mut header_parts = header.splitn(2, ' ');
        let type_ = header_parts
            .next()
            .ok_or(anyhow::anyhow!("Could not find type"))?;
        let byte_size = header_parts
            .next()
            .ok_or(anyhow::anyhow!("Could not find byte size"))?;

        let type_ = ObjectType::from_str(type_)?;
        let byte_size = usize::from_str(byte_size)?;

        if content.len() != byte_size {
            bail!("Byte size did not match content length");
        }

        Ok(Self::new(type_, content.to_vec()))
    }
}
