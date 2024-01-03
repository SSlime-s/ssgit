use std::str::FromStr;

use crate::object::zip::decompress;
use crate::object::{hash::Hash, GitObject};
use crate::parser::CatFileArgs;
use anyhow::{bail, Result};

pub fn handle(args: &CatFileArgs) -> Result<()> {
    dbg!(args);

    let hash = Hash::from_str(&args.hash)?;

    let path_buf = std::path::Path::new(".git/objects")
        .join(hash.directory())
        .join(hash.file());
    let objects_path = path_buf.to_str().ok_or(anyhow::anyhow!("Invalid path"))?;

    if args.options.exists {
        if std::path::Path::new(objects_path).exists() {
            return Ok(());
        } else {
            bail!("")
        }
    }

    let content = std::fs::read(objects_path)?;
    let decompressed_content = decompress(&content)?;

    let objects = GitObject::try_from(decompressed_content.as_slice())?;

    match (args.options.pretty, args.options.type_, args.options.size) {
        (true, _, _) => {
            println!("{}", std::str::from_utf8(&objects.content)?);
        }
        (_, true, _) => {
            println!("{}", objects.type_.to_string());
        }
        (_, _, true) => {
            println!("{}", objects.size());
        }
        _ => unreachable!(),
    }

    Ok(())
}
