use std::str::FromStr;

use crate::entity::object::ObjectType;
use crate::entity::object::{hash::Hash, GitObject};
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

    let objects = GitObject::read(&hash)?;

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
                println!("{}", objects.parse_commit_body()?);
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
