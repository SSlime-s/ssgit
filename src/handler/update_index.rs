use std::{path::Path, str::FromStr};

use crate::{
    index::{Index, IndexEntry},
    object::{hash::Hash, mode::Mode, GitObject},
    parser::UpdateIndexArgs,
};
use anyhow::{bail, Result};

pub fn handle(args: &UpdateIndexArgs) -> Result<()> {
    dbg!(args);

    if args.remove && args.add {
        bail!("Cannot use --add and --remove together")
    }

    if args.remove {
        unimplemented!();
    }

    let mut index = if let Some(index) = Index::read()? {
        index
    } else {
        Index::new()
    };

    let mut entries = Vec::with_capacity(args.file.len() + 1);
    if let Some(cache_info) = &args.cacheinfo {
        if cache_info.len() != 3 {
            bail!("cacheinfo should have 3 arguments");
        }

        let mode = Mode::from_str(&cache_info[0])?;
        let hash = Hash::from_str(&cache_info[1])?;
        let file_name = &cache_info[2];

        let file_path = Path::new(file_name);
        let entry = if file_path.exists() {
            IndexEntry::with_default(mode, hash, file_name)
        } else {
            let metadata = file_path.metadata()?;
            IndexEntry::with_file_metadata(mode, hash, file_name, &metadata)?
        };

        entries.push(entry);
    }

    for file_name in &args.file {
        let file_path = Path::new(file_name);
        if !file_path.exists() {
            bail!("File {} does not exist", file_name);
        }

        let metadata = file_path.metadata()?;
        if metadata.is_dir() {
            bail!("Cannot add directory {}", file_name);
        }

        let mode = Mode::from_file_metadata(&metadata)?;

        let content = std::fs::read(file_name)?;
        let hash = GitObject::new(crate::object::ObjectType::Blob, content).hash();

        let entry = IndexEntry::with_file_metadata(mode, hash, file_name, &metadata)?;

        entries.push(entry);
    }

    if !args.add {
        let hash_map = index.hash_map();

        if let Some(entry) = entries
            .iter()
            .find(|e| !hash_map.contains_key(e.file_name.as_str()))
        {
            bail!(
                "File {} is not in the index. Use --add to add it",
                entry.file_name
            );
        }
    }

    index.insert(&entries);

    index.write()?;

    Ok(())
}
