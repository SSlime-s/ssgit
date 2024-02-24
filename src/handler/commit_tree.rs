use std::str::FromStr;

use anyhow::Result;

use crate::{
    entity::object::{hash::Hash, GitObject, User},
    parser::CommitTreeArgs,
};

pub fn handle(args: &CommitTreeArgs) -> Result<()> {
    let message = if args.message.is_empty() {
        std::io::stdin()
            .lines()
            .map(|line| line.map_err(|e| e.into()))
            .collect::<Result<String>>()?
    } else {
        args.message.join("\n\n")
    };

    let now = chrono::Local::now();
    let now = now.with_timezone(now.offset());

    let committer = User::read_from_git(now)?;

    let tree_hash = Hash::from_str(&args.tree)?;

    let commit = crate::entity::object::Commit {
        tree: tree_hash,
        parent: args
            .parent
            .iter()
            .map(|s| Hash::from_str(s))
            .collect::<Result<Vec<_>>>()?,
        author: committer.clone(),
        committer,
        rest_of_header: "".to_string(),
        message,
    };

    let git_object = GitObject::from_commit(&commit);

    let hash = git_object.hash();

    let path = hash.get_object_path();
    if path.exists() {
        let existing_content = std::fs::read(&path)?;
        dbg!(existing_content == git_object.to_raw()?);
        println!("{}", hash);
        return Ok(());
    }

    git_object.write()?;

    println!("{}", hash);

    Ok(())
}
