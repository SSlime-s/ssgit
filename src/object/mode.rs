use std::{str::FromStr, fs::Metadata, os::unix::fs::PermissionsExt};

use anyhow::{bail, Result};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlobType {
    Plain,
    Executable,
    Symlink,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Blob(BlobType),
    Tree,
}
impl FromStr for Mode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "40000" => Ok(Self::Tree),
            "100644" => Ok(Self::Blob(BlobType::Plain)),
            "100755" => Ok(Self::Blob(BlobType::Executable)),
            "120000" => Ok(Self::Blob(BlobType::Symlink)),
            _ => bail!("Invalid object type {}", s),
        }
    }
}
impl TryFrom<u32> for Mode {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0o40000 => Ok(Self::Tree),
            0o100644 => Ok(Self::Blob(BlobType::Plain)),
            0o100755 => Ok(Self::Blob(BlobType::Executable)),
            0o120000 => Ok(Self::Blob(BlobType::Symlink)),
            _ => bail!("Invalid object type {}", value),
        }
    }
}
impl From<Mode> for u32 {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Tree => 0o40000,
            Mode::Blob(BlobType::Plain) => 0o100644,
            Mode::Blob(BlobType::Executable) => 0o100755,
            Mode::Blob(BlobType::Symlink) => 0o120000,
        }
    }
}
impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blob(BlobType::Plain) => write!(f, "100644"),
            Self::Blob(BlobType::Executable) => write!(f, "100755"),
            Self::Blob(BlobType::Symlink) => write!(f, "120000"),
            Self::Tree => write!(f, "40000"),
        }
    }
}
impl Mode {
    pub fn to_format_with_name(self) -> String {
        match self {
            Self::Blob(_) => format!("{:06} blob", self),
            Self::Tree => format!("{:06} tree", self),
        }
    }

    pub fn from_file_metadata(metadata: &Metadata) -> Result<Self> {
        if metadata.is_dir() {
            bail!("Cannot determine mode for directory");
        }

        let mode = metadata.permissions().mode();
        // user に実行権限があるかどうか
        let is_executable = (mode & 0o100) != 0;

        if is_executable {
            Ok(Self::Blob(BlobType::Executable))
        } else {
            Ok(Self::Blob(BlobType::Plain))
        }
    }
}
