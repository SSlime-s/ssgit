use std::str::FromStr;

use crate::{
    entity::{
        head::Head,
        index::Index,
        object::{hash::Hash, GitObject, User},
        refs::Ref,
        tree::TreeNode,
    },
    parser::CommitArgs,
};
use anyhow::Result;

pub fn handle(args: &CommitArgs) -> Result<()> {
    dbg!(args);

    let index = Index::read()?;
    let node = TreeNode::from(index.unwrap());
    let tree_hash = node.write_recursive()?;

    let now = chrono::Local::now();
    let now = now.with_timezone(now.offset());

    let committer = User::read_from_git(now)?;

    let head = Head::read()?;
    if let Head::Detached(_) = head {
        println!("Detached HEAD is not supported");
        return Ok(());
    }
    let Head::Ref(ref_) = head else {
        unreachable!();
    };
    if let Ref::Tag(_) = ref_ {
        println!("Tag is not supported");
        return Ok(());
    }

    let parent = ref_
        .read_hash()?
        .map(|hash| Hash::from_str(&hash))
        .transpose()?;

    let commit = crate::entity::object::Commit {
        tree: tree_hash,
        parent: parent.into_iter().collect(),
        author: committer.clone(),
        committer,
        rest_of_header: "".to_string(),
        message: args.message.clone(),
    };

    let git_object = GitObject::from_commit(&commit);
    let path = git_object.hash().get_object_path();
    if !path.exists() {
        git_object.write()?;
    }

    ref_.write_hash(&git_object.hash().to_string())?;

    Ok(())
}
