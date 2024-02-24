use std::str::FromStr;

use crate::{
    entity::{head::Head, object::hash::Hash, refs::Ref},
    parser::UpdateRefArgs,
};

use anyhow::{bail, Result};

#[derive(Debug, Clone)]
enum RefType {
    Head,
    Ref(Ref),
}
impl FromStr for RefType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == "HEAD" {
            Ok(RefType::Head)
        } else {
            Ok(RefType::Ref(s.parse()?))
        }
    }
}

pub fn handle(args: &UpdateRefArgs) -> Result<()> {
    let ref_type: RefType = args.ref_.parse()?;
    let new_hash: Hash = args.newvalue.parse()?;

    match ref_type {
        RefType::Head => {
            let head = Head::read()?;

            match head {
                Head::Detached(_) => {
                    let new_head = Head::Detached(new_hash);
                    new_head.write()?;
                }
                Head::Ref(ref_) => {
                    let path = ref_.to_path();
                    if path.is_dir() {
                        bail!("Ref is a directory: {}", ref_.to_string());
                    }
                    if !path.exists() {
                        std::fs::create_dir_all(path.parent().unwrap())?;
                    }

                    std::fs::write(path, new_hash.to_string())?;
                }
            }
        }
        RefType::Ref(ref_) => {
            let path = ref_.to_path();
            if path.is_dir() {
                bail!("Ref is a directory: {}", ref_.to_string());
            }
            if !path.exists() {
                std::fs::create_dir_all(path.parent().unwrap())?;
            }

            std::fs::write(path, new_hash.to_string())?;
        }
    };

    Ok(())
}
