use crate::{
    entity::{index::Index, tree::TreeNode},
    parser::WriteTreeArgs,
};
use anyhow::Result;

pub fn handle(args: &WriteTreeArgs) -> Result<()> {
    dbg!(args);

    let index = Index::read()?;

    let node = TreeNode::from(index.unwrap());

    let hash = node.write_recursive()?;

    println!("{}", hash);

    Ok(())
}
