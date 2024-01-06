use std::str::FromStr;

use crate::object::ObjectType;
use crate::object::{hash::Hash, GitObject};
use crate::parser::CatFileArgs;
use anyhow::{bail, Result};

pub fn handle(args: &CatFileArgs) -> Result<()> {
    dbg!(args);

    let hash = Hash::from_str(&args.hash)?;

    let path = hash.get_object_path();

    if args.options.exists {
        if path.exists() {
            return Ok(());
        } else {
            bail!("")
        }
    }

    let content = std::fs::read(path)?;
    let objects = GitObject::from_raw(&content)?;

    match (args.options.pretty, args.options.type_, args.options.size) {
        (true, _, _) => match objects.type_ {
            ObjectType::Blob => {
                println!("{}", objects.parse_blob_body()?);
            }
            ObjectType::Tree => {
                for entry in objects.parse_tree_body()? {
                    println!("{}", entry);
                }
            }
            ObjectType::Commit => {
                unimplemented!();
            }
        },
        (_, true, _) => {
            println!("{}", objects.type_.to_string());
        }
        (_, _, true) => {
            println!("{}", objects.size());
        }
        _ => unreachable!(),
    }

    dbg!(objects.hash());

    Ok(())
}
