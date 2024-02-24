use std::{
    fmt::{Display, Write},
    os::unix::fs::PermissionsExt,
    str::FromStr,
};

use crate::parser::ObjectType as ParserObjectType;
use anyhow::{bail, Result};
use chrono::{FixedOffset, TimeZone};

use self::{
    mode::Mode,
    zip::{compress, decompress},
};

pub mod hash;
pub mod mode;
pub mod zip;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeEntry {
    pub file_type: Mode,
    pub name: String,
    pub hash: hash::Hash,
}
impl TreeEntry {
    fn sort_key(&self) -> String {
        match self.file_type {
            Mode::Tree => format!("{}/", self.name),
            Mode::Blob(_) => self.name.clone(),
        }
    }
}
impl std::fmt::Display for TreeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}\t{}",
            self.file_type.to_format_with_name(),
            self.hash,
            self.name
        )
    }
}
impl PartialOrd for TreeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for TreeEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
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

    pub fn read(hash: &hash::Hash) -> Result<Self> {
        let path = hash.get_object_path();

        let content = std::fs::read(path)?;

        Self::from_raw(&content)
    }

    pub fn write(&self) -> Result<()> {
        let hash = self.hash();
        let path = hash.get_object_path();

        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, self.to_raw()?)?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444))?;

        Ok(())
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
            let mode = Mode::from_str(mode)?;

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

    pub fn parse_commit_body(&self) -> Result<Commit> {
        if self.type_ != ObjectType::Commit {
            panic!("Object is not a commit");
        }

        let commit = Commit::from_str(std::str::from_utf8(&self.body)?)?;

        Ok(commit)
    }

    pub fn from_commit(commit: &Commit) -> Self {
        let body = commit.to_string().as_bytes().to_vec();

        Self::new(ObjectType::Commit, body)
    }

    pub fn new_tree(entries: &[TreeEntry]) -> Self {
        let mut entries = entries.to_vec();
        entries.sort();
        let mut bytes = Vec::new();

        for entry in entries {
            bytes.extend_from_slice(entry.file_type.to_string().as_bytes());
            bytes.push(b' ');
            bytes.extend_from_slice(entry.name.as_bytes());
            bytes.push(b'\0');
            bytes.extend_from_slice(&entry.hash.to_raw());
        }

        Self::new(ObjectType::Tree, bytes)
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Commit {
    pub tree: hash::Hash,
    pub parent: Vec<hash::Hash>,
    pub author: User,
    pub committer: User,
    pub rest_of_header: String,
    pub message: String,
}
impl FromStr for Commit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, "\n\n");

        let header = parts
            .next()
            .ok_or(anyhow::anyhow!("Could not find header"))?;
        let message = parts
            .next()
            .ok_or(anyhow::anyhow!("Could not find message"))?;

        let header_lines = header.lines().collect::<Vec<&str>>();
        let mut header_lines = header_lines.iter().peekable();

        let tree_line = header_lines
            .next()
            .ok_or(anyhow::anyhow!("Could not find tree line"))?;
        let tree = hash::Hash::from_str(
            tree_line
                .strip_prefix("tree ")
                .ok_or(anyhow::anyhow!("Could not find tree"))?,
        )?;

        let parent = {
            let mut parent = Vec::new();

            while header_lines
                .peek()
                .ok_or(anyhow::anyhow!("Could not find parent line"))?
                .starts_with("parent ")
            {
                let parent_line = header_lines
                    .next()
                    .ok_or(anyhow::anyhow!("Could not find parent line"))?;
                let parent_hash = hash::Hash::from_str(
                    parent_line
                        .strip_prefix("parent ")
                        .ok_or(anyhow::anyhow!("Could not find parent"))?,
                )?;
                parent.push(parent_hash);
            }

            parent
        };

        let author_line = header_lines
            .next()
            .ok_or(anyhow::anyhow!("Could not find author line"))?;
        let author = User::from_str(
            author_line
                .strip_prefix("author ")
                .ok_or(anyhow::anyhow!("Could not find author"))?,
        )?;

        let committer_line = header_lines
            .next()
            .ok_or(anyhow::anyhow!("Could not find committer line"))?;
        let committer = User::from_str(
            committer_line
                .strip_prefix("committer ")
                .ok_or(anyhow::anyhow!("Could not find committer"))?,
        )?;

        let rest_of_header = header_lines.copied().collect::<Vec<&str>>().join("\n");

        Ok(Self {
            tree,
            parent,
            author,
            committer,
            rest_of_header,
            message: message.to_string(),
        })
    }
}
impl Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parent = self.parent.iter().fold(String::new(), |mut acc, parent| {
            let _ = writeln!(acc, "parent {}", parent);
            acc
        });
        let _ = self
            .parent
            .iter()
            .map(|parent| format!("parent {}\n", parent))
            .collect::<Vec<String>>()
            .join("");

        write!(
            f,
            "tree {}\n{}author {}\ncommitter {}\n{}\n\n{}",
            self.tree, parent, self.author, self.committer, self.rest_of_header, self.message
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct User {
    pub name: String,
    pub email: String,
    pub time: chrono::DateTime<FixedOffset>,
}
impl User {
    pub fn new(name: String, email: String, time: chrono::DateTime<FixedOffset>) -> Self {
        Self { name, email, time }
    }

    pub fn read_from_git(time: chrono::DateTime<FixedOffset>) -> Result<Self> {
        let username_child = std::process::Command::new("git")
            .args(["config", "--get", "user.name"])
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        let email_child = std::process::Command::new("git")
            .args(["config", "--get", "user.email"])
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        let username_raw = username_child.wait_with_output()?.stdout;
        let email_raw = email_child.wait_with_output()?.stdout;

        let username = String::from_utf8_lossy(&username_raw).trim().to_string();
        let email = String::from_utf8_lossy(&email_raw).trim().to_string();

        Ok(Self::new(username, email, time))
    }
}
impl FromStr for User {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(4, ' ');

        let name = parts
            .next()
            .ok_or(anyhow::anyhow!("Could not find name"))?
            .to_string();
        let email = parts
            .next()
            .and_then(|s| s.strip_prefix('<'))
            .and_then(|s| s.strip_suffix('>'))
            .ok_or(anyhow::anyhow!("Could not find email"))?
            .to_string();
        let timestamp = parts
            .next()
            .ok_or(anyhow::anyhow!("Could not find timestamp"))?;
        let offset = parts
            .next()
            .ok_or(anyhow::anyhow!("Could not find offset"))?;

        let offset = FixedOffset::from_str(format!("{}:{}", &offset[..3], &offset[3..]).as_str())?;
        let time = offset
            .timestamp_micros(i64::from_str(timestamp)?)
            .single()
            .ok_or(anyhow::anyhow!("Could not parse timestamp"))?;

        Ok(Self::new(name, email, time))
    }
}
impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} <{}> {} {}",
            self.name,
            self.email,
            self.time.timestamp_micros(),
            format!("{}", self.time.offset()).replace(':', "")
        )
    }
}
