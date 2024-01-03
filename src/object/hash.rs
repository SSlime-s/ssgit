use std::str::FromStr;

use anyhow::{bail, Result};
use sha1::{Digest, Sha1};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Hash {
    value: String,
}
impl FromStr for Hash {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 40 {
            bail!("Hash must be 40 characters long");
        }

        Ok(Self {
            value: s.to_string(),
        })
    }
}
impl From<[u8; 20]> for Hash {
    fn from(bytes: [u8; 20]) -> Self {
        Self {
            value: std::str::from_utf8(&bytes).unwrap().to_string(),
        }
    }
}
impl Hash {
    pub fn new(value: &str) -> Result<Self> {
        Self::from_str(value)
    }

    pub fn directory(&self) -> &str {
        &self.value[..2]
    }

    pub fn file(&self) -> &str {
        &self.value[2..]
    }

    #[allow(clippy::self_named_constructors)]
    pub fn hash(content: &str) -> Self {
        let hash: [u8; 20] = Sha1::digest(content.as_bytes()).into();

        Self::from(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_HASH: &str = "0123456789012345678901234567890123456789";
    const SHORT_HASH: &str = "012345678901234567890123456789012345678";

    #[test]
    fn test_hash_from_str() {
        let hash = Hash::from_str(VALID_HASH).unwrap();

        assert_eq!(hash.value, VALID_HASH);
    }

    #[test]
    fn test_hash_from_str_short() {
        let hash = Hash::from_str(SHORT_HASH);

        assert!(hash.is_err());
    }

    #[test]
    fn test_hash_directory() {
        let hash = Hash::from_str(VALID_HASH).unwrap();

        assert_eq!(hash.directory(), "01");
    }

    #[test]
    fn test_hash_file() {
        let hash = Hash::from_str(VALID_HASH).unwrap();

        assert_eq!(hash.file(), "23456789012345678901234567890123456789");
    }
}