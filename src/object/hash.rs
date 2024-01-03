use std::str::FromStr;

use anyhow::{bail, Result};
use sha1::{Digest, Sha1};

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("")
}

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        bail!("Hex string must be even length");
    }

    let mut bytes = Vec::new();

    for i in 0..(hex.len() / 2) {
        let start = i * 2;
        let end = start + 2;
        let hex_byte = &hex[start..end];
        let byte = u8::from_str_radix(hex_byte, 16)?;
        bytes.push(byte);
    }

    Ok(bytes)
}

pub fn hex_to_fixed_bytes<const N: usize>(hex: &str) -> Result<[u8; N]> {
    let bytes = hex_to_bytes(hex)?;

    if bytes.len() != N {
        bail!("Hex string must be {} bytes long", N);
    }

    let mut fixed_bytes = [0; N];
    fixed_bytes.copy_from_slice(&bytes);

    Ok(fixed_bytes)
}

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
            value: bytes_to_hex(&bytes),
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
        Self::hash_bytes(content.as_bytes())
    }

    pub fn hash_bytes(content: &[u8]) -> Self {
        let hash: [u8; 20] = Sha1::digest(content).into();

        Self::from(hash)
    }
}
impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
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
