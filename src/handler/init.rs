use std::path::Path;

use crate::{consts, parser::InitArgs};

use anyhow::Result;

pub fn handle(args: &InitArgs) -> Result<()> {
    let git_path = Path::new(consts::GIT_DIRECTORY);

    if git_path.exists() {
        println!("already initialized");
        return Ok(());
    }

    std::fs::create_dir(git_path)?;

    let refs_path = Path::new(consts::REFS_DIRECTORY);
    let refs_heads_path = refs_path.join("heads");
    let refs_tags_path = refs_path.join("tags");

    let objects_path = Path::new(consts::OBJECTS_DIRECTORY);

    std::fs::create_dir(refs_path)?;
    std::fs::create_dir(refs_heads_path)?;
    std::fs::create_dir(refs_tags_path)?;

    std::fs::create_dir(objects_path)?;

    let head_path = Path::new(consts::HEAD_PATH);
    let branch = args
        .initial_branch
        .clone()
        .unwrap_or(consts::DEFAULT_BRANCH.to_string());
    std::fs::write(head_path, format!("ref: refs/heads/{}", branch))?;

    Ok(())
}
