use std::collections::HashMap;
use std::path::PathBuf;

use crate::index::Index;
use crate::object::hash::Hash;
use crate::object::mode::{BlobType, Mode};
use crate::object::{GitObject, TreeEntry};
use anyhow::{bail, Result};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TreeNodeInfo {
    Blob(BlobType, Hash),
    Tree(HashMap<String, TreeNode>),
}
impl TreeNodeInfo {
    fn new_tree() -> Self {
        TreeNodeInfo::Tree(HashMap::new())
    }

    fn new_blob(blob_type: BlobType, hash: Hash) -> Self {
        TreeNodeInfo::Blob(blob_type, hash)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum TraverseNode<'a, ReturnValue> {
    Blob(&'a TreeNode),
    Tree(&'a TreeNode, HashMap<String, ReturnValue>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeNode {
    pub tree_info: TreeNodeInfo,
    pub name: String,
}
impl TreeNode {
    fn new_blob(blob_type: BlobType, name: String, hash: Hash) -> Self {
        TreeNode {
            tree_info: TreeNodeInfo::new_blob(blob_type, hash),
            name,
        }
    }

    fn new_tree(name: String) -> Self {
        TreeNode {
            tree_info: TreeNodeInfo::new_tree(),
            name,
        }
    }

    pub fn new_root() -> Self {
        TreeNode {
            tree_info: TreeNodeInfo::new_tree(),
            name: "".to_string(),
        }
    }

    fn push_node(&mut self, name: String, node: TreeNode) -> Result<()> {
        match &mut self.tree_info {
            TreeNodeInfo::Tree(tree) => {
                tree.insert(name, node);
                Ok(())
            }
            _ => bail!("Cannot push node to non-tree node"),
        }
    }

    fn push_vec_path(&mut self, mut path: Vec<String>, node: TreeNodeInfo) -> Result<()> {
        let name = path.remove(0);
        if path.is_empty() {
            self.push_node(
                name.clone(),
                TreeNode {
                    tree_info: node,
                    name,
                },
            )?;

            return Ok(());
        }

        match &mut self.tree_info {
            TreeNodeInfo::Tree(tree) => {
                let next_node = tree
                    .entry(name.clone())
                    .or_insert_with(|| TreeNode::new_tree(name.clone()));

                next_node.push_vec_path(path, node)
            }
            _ => bail!("Cannot push node to non-tree node"),
        }
    }

    pub fn push_full_path_blob(
        &mut self,
        path: impl Into<PathBuf>,
        blob_type: BlobType,
        hash: Hash,
    ) -> Result<()> {
        let path: PathBuf = path.into();

        if path.is_absolute() {
            bail!("Path must be relative")
        }

        if path.is_dir() {
            bail!("Path must be a file")
        }

        let path = path
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        self.push_vec_path(path, TreeNodeInfo::new_blob(blob_type, hash))
    }

    fn post_order_traversal<ReturnValue>(
        &self,
        mut f: impl FnMut(TraverseNode<ReturnValue>) -> ReturnValue,
    ) -> ReturnValue {
        match &self.tree_info {
            TreeNodeInfo::Blob(_, _) => f(TraverseNode::Blob(self)),
            TreeNodeInfo::Tree(tree) => {
                let mut children = HashMap::new();

                for (name, node) in tree {
                    children.insert(name.clone(), node.post_order_traversal(&mut f));
                }

                f(TraverseNode::Tree(self, children))
            }
        }
    }

    fn post_order_traversal_or_err<ReturnValue>(
        &self,
        f: &mut impl FnMut(TraverseNode<ReturnValue>) -> Result<ReturnValue>,
    ) -> Result<ReturnValue> {
        match &self.tree_info {
            TreeNodeInfo::Blob(_, _) => f(TraverseNode::Blob(self)),
            TreeNodeInfo::Tree(tree) => {
                let mut children = HashMap::new();

                for (name, node) in tree {
                    children.insert(name.clone(), node.post_order_traversal_or_err(f)?);
                }

                f(TraverseNode::Tree(self, children))
            }
        }
    }

    fn write_recursive_inner(traverse_node: TraverseNode<(Mode, Hash)>) -> Result<(Mode, Hash)> {
        match traverse_node {
            TraverseNode::Blob(node) => {
                let (blob_type, hash) = match &node.tree_info {
                    TreeNodeInfo::Blob(blob_type, hash) => (blob_type, hash),
                    _ => unreachable!("Expected blob node"),
                };

                let mode = Mode::Blob(*blob_type);
                Ok((mode, hash.clone()))
            }
            TraverseNode::Tree(_node, children) => {
                let mut tree_entries = Vec::new();

                for (name, (mode, hash)) in children {
                    let entry = TreeEntry {
                        file_type: mode,
                        name,
                        hash,
                    };

                    tree_entries.push(entry);
                }

                let git_object = GitObject::new_tree(&tree_entries);
                let hash = git_object.hash();

                let path = hash.get_object_path();
                if path.exists() {
                    let existing_object = GitObject::read(&hash)?;
                    dbg!(existing_object == git_object);
                    if existing_object != git_object {
                        dbg!(&path);
                    }
                    return Ok((Mode::Tree, hash));
                }

                git_object.write()?;

                let mode = Mode::Tree;

                Ok((mode, hash))
            }
        }
    }

    pub fn write_recursive(&self) -> Result<Hash> {
        let (_mode, hash) = self.post_order_traversal_or_err(&mut Self::write_recursive_inner)?;

        Ok(hash)
    }
}
impl From<Index> for TreeNode {
    fn from(index: Index) -> Self {
        let mut root = Self::new_root();

        for entry in index.entries {
            let Mode::Blob(blob_type) = entry.mode else {
                panic!("Unexpected tree entry in index: {:?}", entry);
            };

            root.push_full_path_blob(&entry.file_name, blob_type, entry.hash)
                .unwrap();
        }

        root
    }
}
