use std::{collections::HashMap, os::unix::fs::MetadataExt, path::Path};

use crate::{
    consts::GIT_INDEX_PATH,
    object::{hash::Hash, mode::Mode},
};
use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index {
    pub version: u32,
    pub entries: Vec<IndexEntry>,
}
impl Index {
    pub fn new() -> Self {
        Self {
            version: 2,
            entries: Vec::new(),
        }
    }

    pub fn read() -> Result<Option<Self>> {
        if !Path::new(GIT_INDEX_PATH).try_exists().unwrap_or(true) {
            return Ok(None);
        }
        let bytes = std::fs::read(GIT_INDEX_PATH)?;
        Self::from_raw(&bytes).map(Some)
    }

    pub fn write(&self) -> Result<()> {
        let bytes = self.to_raw();
        std::fs::write(GIT_INDEX_PATH, bytes)?;
        Ok(())
    }

    pub fn from_raw(bytes: &[u8]) -> Result<Self> {
        let mut shown_index = 0;

        assert_eq!(bytes[..4], b"DIRC"[..]);

        let version = BigEndian::read_u32(&bytes[4..8]);

        let entry_count = BigEndian::read_u32(&bytes[8..12]) as usize;

        dbg!(version, entry_count);

        let mut entries = Vec::with_capacity(entry_count);

        shown_index += 12;

        for _ in 0..entry_count {
            let entry = IndexEntry::from_raw(bytes, &mut shown_index)?;
            entries.push(entry);
        }

        Ok(Self { version, entries })
    }

    pub fn to_raw(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"DIRC");
        bytes.extend_from_slice(&self.version.to_be_bytes());
        bytes.extend_from_slice(&(self.entries.len() as u32).to_be_bytes());

        for entry in &self.entries {
            bytes.extend_from_slice(&entry.to_raw());
        }

        bytes
    }

    pub fn insert(&mut self, entries: &[IndexEntry]) {
        let mut hash_map = self.hash_map();

        for entry in entries {
            hash_map.insert(entry.file_name.as_str(), entry);
        }

        self.entries = hash_map.into_values().cloned().collect::<Vec<_>>();
        self.sort();
    }

    fn sort(&mut self) {
        self.entries.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    }

    pub fn hash_map(&self) -> HashMap<&str, &IndexEntry> {
        self.entries
            .iter()
            .map(|e| (e.file_name.as_str(), e))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    pub created_at: u32,
    pub created_at_nsec: u32,
    pub updated_at: u32,
    pub updated_at_nsec: u32,
    pub device_id: u32,
    pub inode: u32,
    pub mode: Mode,
    pub user_id: u32,
    pub group_id: u32,
    pub size: u32,
    pub hash: Hash,
    pub file_name: String,
}
impl IndexEntry {
    pub fn with_default(mode: Mode, hash: Hash, file_name: &str) -> Self {
        Self {
            created_at: 0,
            created_at_nsec: 0,
            updated_at: 0,
            updated_at_nsec: 0,
            device_id: 0,
            inode: 0,
            mode,
            user_id: 0,
            group_id: 0,
            size: 0,
            hash,
            file_name: file_name.to_string(),
        }
    }

    pub fn with_file_metadata(
        mode: Mode,
        hash: Hash,
        file_name: &str,
        metadata: &std::fs::Metadata,
    ) -> Result<Self> {
        let created_at = metadata.ctime() as u32;
        let created_at_nsec = metadata.ctime_nsec() as u32;
        let updated_at = metadata.mtime() as u32;
        let updated_at_nsec = metadata.mtime_nsec() as u32;
        let device_id = metadata.dev() as u32;
        let inode = metadata.ino() as u32;
        let user_id = metadata.uid();
        let group_id = metadata.gid();
        let size = metadata.size() as u32;

        Ok(Self {
            created_at,
            created_at_nsec,
            updated_at,
            updated_at_nsec,
            device_id,
            inode,
            mode,
            user_id,
            group_id,
            size,
            hash,
            file_name: file_name.to_string(),
        })
    }

    pub fn from_raw(bytes: &[u8], shown_index: &mut usize) -> Result<Self> {
        let bytes = &bytes[*shown_index..];
        let created_at = BigEndian::read_u32(&bytes[..4]);
        let created_at_nsec = BigEndian::read_u32(&bytes[4..8]);
        let updated_at = BigEndian::read_u32(&bytes[8..12]);
        let updated_at_nsec = BigEndian::read_u32(&bytes[12..16]);
        let device_id = BigEndian::read_u32(&bytes[16..20]);
        let inode = BigEndian::read_u32(&bytes[20..24]);
        let mode: Mode = BigEndian::read_u32(&bytes[24..28]).try_into()?;
        let user_id = BigEndian::read_u32(&bytes[28..32]);
        let group_id = BigEndian::read_u32(&bytes[32..36]);
        let size = BigEndian::read_u32(&bytes[36..40]);
        let hash = Hash::from_raw(&bytes[40..60])?;

        let file_name_length = BigEndian::read_u16(&bytes[60..62]) as usize;
        let file_name = std::str::from_utf8(&bytes[62..62 + file_name_length])?.to_string();

        let entry_length = 62 + file_name_length;
        let padding = 8 - (entry_length % 8);

        let entry_length = entry_length + padding;
        *shown_index += entry_length;

        Ok(Self {
            created_at,
            created_at_nsec,
            updated_at,
            updated_at_nsec,
            device_id,
            inode,
            mode,
            user_id,
            group_id,
            size,
            hash,
            file_name,
        })
    }

    pub fn to_raw(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.created_at.to_be_bytes());
        bytes.extend_from_slice(&self.created_at_nsec.to_be_bytes());
        bytes.extend_from_slice(&self.updated_at.to_be_bytes());
        bytes.extend_from_slice(&self.updated_at_nsec.to_be_bytes());
        bytes.extend_from_slice(&self.device_id.to_be_bytes());
        bytes.extend_from_slice(&self.inode.to_be_bytes());
        bytes.extend_from_slice(&u32::from(self.mode).to_be_bytes());
        bytes.extend_from_slice(&self.user_id.to_be_bytes());
        bytes.extend_from_slice(&self.group_id.to_be_bytes());
        bytes.extend_from_slice(&self.size.to_be_bytes());
        bytes.extend_from_slice(&self.hash.to_raw());
        bytes.extend_from_slice(&(self.file_name.len() as u16).to_be_bytes());
        bytes.extend_from_slice(self.file_name.as_bytes());

        let padding = 8 - (bytes.len() % 8);
        bytes.extend_from_slice(&vec![0; padding]);

        bytes
    }
}
