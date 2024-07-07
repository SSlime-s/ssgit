use std::path::Path;

use crate::{
    entity::{
        index::{Index, IndexEntry},
        object::{mode::Mode, GitObject, ObjectType},
    },
    parser::AddArgs,
};
use anyhow::{bail, Result};

pub fn handle(args: &AddArgs) -> Result<()> {
    dbg!(args);

    if args.files.is_empty() {
        println!("Nothing specified, nothing added.");
        return Ok(());
    }

    let mut entries = Vec::with_capacity(args.files.len());
    for file_name in &args.files {
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
        let object = GitObject::new(ObjectType::Blob, content);
        let hash = object.hash();

        if !hash.get_object_path().exists() {
            object.write()?;
        }

        let entry = IndexEntry::with_file_metadata(mode, hash, file_name, &metadata)?;

        entries.push(entry);
    }

    let mut index = if let Some(index) = Index::read()? {
        index
    } else {
        Index::new()
    };

    index.insert(&entries);

    index.write()?;

    Ok(())
}
