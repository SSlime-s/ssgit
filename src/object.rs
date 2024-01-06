use std::str::FromStr;

use crate::parser::ObjectType as ParserObjectType;
use anyhow::{bail, Result};

use self::zip::{decompress, compress};

pub mod hash;
pub mod zip;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeEntry {
    pub file_type: TreeEntryType,
    pub name: String,
    pub hash: hash::Hash,
}
impl std::fmt::Display for TreeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}\t{}", self.file_type, self.hash, self.name)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TreeEntryType {
    Blob,
    Tree,
}
impl TryFrom<&str> for TreeEntryType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "40000" => Ok(Self::Tree),
            "100644" => Ok(Self::Blob),
            _ => bail!("Invalid tree entry type {}", value),
        }
    }
}
impl std::fmt::Display for TreeEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeEntryType::Blob => write!(f, "100644 blob"),
            TreeEntryType::Tree => write!(f, "040000 tree"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}
impl From<ParserObjectType> for ObjectType {
    fn from(parser_object_type: ParserObjectType) -> Self {
        match parser_object_type {
            ParserObjectType::Blob => Self::Blob,
            ParserObjectType::Tree => Self::Tree,
            ParserObjectType::Commit => Self::Commit,
        }
    }
}
impl FromStr for ObjectType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(Self::Blob),
            "tree" => Ok(Self::Tree),
            "commit" => Ok(Self::Commit),
            _ => bail!("Invalid object type {}", s),
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
    pub body: Vec<u8>,
}
impl GitObject {
    pub fn new(type_: ObjectType, body: Vec<u8>) -> Self {
        Self { type_, body }
    }

    pub fn size(&self) -> usize {
        self.body.len()
    }

    pub fn hash(&self) -> hash::Hash {
        let full_content: Vec<u8> = self.into();

        hash::Hash::hash_bytes(&full_content)
    }

    pub fn parse_blob_body(&self) -> Result<String> {
        if self.type_ != ObjectType::Blob {
            bail!("Object is not a blob");
        }

        Ok(std::str::from_utf8(&self.body)?.to_string())
    }

    const TREE_SHA1_LENGTH: usize = 20;
    pub fn parse_tree_body(&self) -> Result<Vec<TreeEntry>> {
        if self.type_ != ObjectType::Tree {
            bail!("Object is not a tree");
        }

        let mut entries = Vec::new();
        let mut body = self.body.clone();

        while !body.is_empty() {
            let mut parts = body.splitn(2, |b: &u8| *b == b' ');

            let mode = parts.next().ok_or(anyhow::anyhow!("Could not find mode"))?;
            let mode = std::str::from_utf8(mode)?;
            let mode = TreeEntryType::try_from(mode)?;

            body = parts
                .next()
                .ok_or(anyhow::anyhow!("unexpected end of body"))?
                .to_vec();

            let mut parts = body.splitn(2, |b: &u8| *b == b'\0');

            let name = parts.next().ok_or(anyhow::anyhow!("Could not find name"))?;
            let name = std::str::from_utf8(name)?;
            let name = name.to_string();

            body = parts
                .next()
                .ok_or(anyhow::anyhow!("unexpected end of body"))?
                .to_vec();

            let hash = body.drain(..Self::TREE_SHA1_LENGTH).collect::<Vec<u8>>();
            let hash: [u8; Self::TREE_SHA1_LENGTH] = hash.try_into().map_err(|_| {
                anyhow::anyhow!("Could not convert hash to [u8; {}]", Self::TREE_SHA1_LENGTH)
            })?;
            let hash = hash::Hash::from(hash);

            entries.push(TreeEntry {
                file_type: mode,
                name,
                hash,
            });
        }

        Ok(entries)
    }

    pub fn from_raw(bytes: &[u8]) -> Result<Self> {
        let decompressed_bytes = decompress(bytes)?;

        Self::try_from(decompressed_bytes.as_slice())
    }

    pub fn to_raw(&self) -> Result<Vec<u8>> {
        let bytes: Vec<u8> = self.into();

        compress(&bytes)
    }
}
impl TryFrom<&[u8]> for GitObject {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut parts = value.splitn(2, |b: &u8| *b == b'\0');

        let header = parts
            .next()
            .ok_or(anyhow::anyhow!("Could not find header"))?;
        let body = parts.next().ok_or(anyhow::anyhow!("Could not find body"))?;

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

        if body.len() != byte_size {
            bail!("Byte size did not match body length");
        }

        Ok(Self::new(type_, body.to_vec()))
    }
}
impl From<&GitObject> for Vec<u8> {
    fn from(object: &GitObject) -> Self {
        let header = format!("{} {}\0", object.type_.to_string(), object.size());
        let mut bytes = header.as_bytes().to_vec();
        bytes.extend(object.body.clone());
        bytes
    }
}
impl From<GitObject> for Vec<u8> {
    fn from(object: GitObject) -> Self {
        Self::from(&object)
    }
}
