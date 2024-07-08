use crate::{
    entity::{head::Head, refs::Ref},
    parser::BranchArgs,
};
use anyhow::{bail, Result};

pub fn handle(args: &BranchArgs) -> Result<()> {
    dbg!(args);

    match (&args.options.create, &args.options.delete) {
        (None, None) => {
            let branches = {
                let mut branches = Ref::branch_names()?;
                branches.sort();
                branches
            };
            let head = Head::read()?;
            let head_branch = match head {
                Head::Ref(ref_) => match ref_ {
                    Ref::Branch(branch) => Some(branch),
                    Ref::Tag(_) => None,
                },
                Head::Detached(_) => None,
            };

            let result = branches
                .iter()
                .map(|branch| {
                    if Some(branch) == head_branch.as_ref() {
                        format!("* {}", branch)
                    } else {
                        format!("  {}", branch)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            println!("{}", result);
        }
        (Some(branch_name), None) => {
            let ref_ = Ref::Branch(branch_name.clone());
            if ref_.to_path().exists() {
                bail!("Branch {} already exists", branch_name);
            }

            let head = Head::read()?;
            let hash = match head {
                Head::Ref(ref_) => match ref_ {
                    Ref::Branch(_) => ref_.read_hash()?,
                    Ref::Tag(_) => bail!("Tag is not supported"),
                },
                Head::Detached(_) => None,
            };

            let Some(hash) = hash else {
                bail!("No commit on branch \"{}\"", branch_name);
            };

            ref_.write_hash(&hash)?;
        },
        (None, Some(branch_name)) => {
            let head = Head::read()?;
            let head_branch = match head {
                Head::Ref(ref_) => match ref_ {
                    Ref::Branch(branch) => Some(branch),
                    Ref::Tag(_) => None,
                },
                Head::Detached(_) => None,
            };

            if Some(branch_name) == head_branch.as_ref() {
                bail!("Cannot delete the branch you are on\nPlease switch to another branch and try again");
            }

            let ref_ = Ref::Branch(branch_name.clone());
            let path = ref_.to_path();
            if !path.exists() {
                bail!("Branch {} does not exist", branch_name);
            }

            std::fs::remove_file(path)?;
        },
        _ => {
            unreachable!()
        }
    }

    Ok(())
}
