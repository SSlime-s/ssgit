use std::str::FromStr;

use crate::{
    entity::{head::Head, object::hash::Hash, refs::Ref},
    parser::SwitchArgs,
};
use anyhow::{bail, Result};

pub fn handle(args: &SwitchArgs) -> Result<()> {
    dbg!(args);

    // TODO: ブランチに応じたファイル内容の変更

    match (
        &args.options.switch,
        &args.options.create,
        &args.options.orphan,
        &args.options.detach,
    ) {
        (Some(branch), None, None, None) => {
            let ref_ = Ref::Branch(branch.clone());
            if !ref_.to_path().exists() {
                bail!("Branch {} does not exist", branch);
            }

            let head = Head::Ref(ref_);
            head.write()?;
        }
        // create
        (None, Some(branch), None, None) => {
            let ref_ = Ref::Branch(branch.clone());
            if ref_.to_path().exists() {
                bail!("Branch {} already exists", branch);
            }

            let current_head = Head::read()?;
            let hash = {
                if let Head::Ref(ref_) = current_head {
                    match &ref_ {
                        Ref::Branch(_) => ref_.read_hash()?,
                        Ref::Tag(_) => {
                            bail!("Tag is not supported");
                        }
                    }
                } else {
                    None
                }
            };

            if let Some(hash) = hash {
                ref_.write_hash(&hash)?;
            }

            let head = Head::Ref(ref_);
            head.write()?;
        }
        // orphan
        (None, None, Some(branch), None) => {
            let ref_ = Ref::Branch(branch.clone());
            if ref_.to_path().exists() {
                bail!("Branch {} already exists", branch);
            }

            let head = Head::Ref(ref_);
            head.write()?;
        }
        // detach
        (None, None, None, Some(hash)) => {
            let hash = Hash::from_str(hash)?;
            let head = Head::Detached(hash);

            head.write()?;
        }
        _ => {
            bail!("Invalid options");
        }
    }

    Ok(())
}
