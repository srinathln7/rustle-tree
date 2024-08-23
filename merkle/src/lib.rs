use log::info;
use std::error::Error;
use std::fmt;
use util::{calc_sha256, max, min};

#[derive(Debug)]
pub struct MerkleTreeError {
    details: String,
}

// Define a `new` method
impl MerkleTreeError {
    fn new(msg: &str) -> MerkleTreeError {
        MerkleTreeError {
            details: (msg.to_string()),
        }
    }
}

// impl: Display trait on MerkleTreeError
impl fmt::Display for MerkleTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MerkleTreeError: {}", self.details)
    }
}

// impl std::error::Error for MerkleTreeError
impl Error for MerkleTreeError {}

#[derive(PartialEq)]
pub struct TreeNode {
    pub hash: String,
    pub left_idx: usize,
    pub right_idx: usize,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
}

pub struct MerkleTree {
    root: Option<Box<TreeNode>>,
}

impl MerkleTree {
    // Constructor for Merkle Tree
    pub fn new(files: &[Vec<u8>]) -> Result<MerkleTree, MerkleTreeError> {
        let n = files.len();
        if n == 0 {
            return Err(MerkleTreeError::new("Empty file list"));
        }

        info!("Creating a new Merkle tree with {} files", files.len());
        let root = MerkleTree::build_tree(files, 0, n - 1);
        Ok(MerkleTree {
            root: Some(Box::new(root)),
        })
    }

    // Recursively build the Merkle tree
    fn build_tree(files: &[Vec<u8>], left: usize, right: usize) -> TreeNode {
        if left == right {
            return TreeNode {
                hash: calc_sha256(&files[left]),
                left_idx: left,
                right_idx: right,
                left: None,
                right: None,
            };
        }

        let mid = left + (right - left) / 2;
        let left_child = MerkleTree::build_tree(files, left, mid);
        let right_child = MerkleTree::build_tree(files, mid + 1, right);

        let combined_hash =
            calc_sha256(format!("{}{}", left_child.hash, right_child.hash).as_bytes());

        TreeNode {
            hash: combined_hash,
            left_idx: left,
            right_idx: right,
            left: Some(Box::new(left_child)),
            right: Some(Box::new(right_child)),
        }
    }

    // GenerateMerkleProof generates a Merkle proof for the given leaf index.
    pub fn generate_merkle_proof(
        &self,
        leaf_idx: usize,
    ) -> Result<Vec<&TreeNode>, MerkleTreeError> {
        info!(
            "[merkle-tree] starting to generate merkle proof for file index {}",
            leaf_idx
        );
        gen_proof(self.root.as_deref().expect("No root"), leaf_idx)
    }
}

// gen_proof generates a Merkle proof for the given leaf index.
pub fn gen_proof(root: &TreeNode, leaf_idx: usize) -> Result<Vec<&TreeNode>, MerkleTreeError> {
    // Check for errors: root bring none or leaf index out of bounds
    if leaf_idx < root.left_idx || leaf_idx > root.right_idx {
        return Err(MerkleTreeError::new("index out of bounds"));
    }

    // If node is a leaf
    if root.left.is_none() && root.right.is_none() {
        return Ok(vec![root]);
    }

    let mut result: Vec<&TreeNode> = Vec::new();

    // Find the sibling of the node at leaf_idx
    let sibling = find_sibling_by_leaf_index(root, leaf_idx)?;
    result.push(sibling);

    // Find the parent of the node at leaf_idx
    let mut parent = find_parent_by_leaf_index(root, leaf_idx)?;

    while parent != root {
        let sibling = find_sibling(root, parent)?;
        result.push(sibling);
        parent = find_parent(root, parent)?;
    }

    Ok(result)
}

// find_sibling_by_leaf_index finds the sibling node of the leaf node corresponding to the given leaf index.
fn find_sibling_by_leaf_index(
    root: &TreeNode,
    leaf_idx: usize,
) -> Result<&TreeNode, MerkleTreeError> {
    let leaf = find_leaf(root, leaf_idx)?;
    find_sibling(root, leaf)
}

// find_parent_by_leaf_index finds the sibling node of the leaf node corresponding to the given leaf index.
fn find_parent_by_leaf_index(
    root: &TreeNode,
    leaf_idx: usize,
) -> Result<&TreeNode, MerkleTreeError> {
    let leaf = find_leaf(root, leaf_idx)?;
    find_parent(root, leaf)
}

//find_leaf finds the leaf node corresponding to the given leaf index.
fn find_leaf(root: &TreeNode, leaf_idx: usize) -> Result<&TreeNode, MerkleTreeError> {
    match root {
        _ if root.left.is_none()
            && root.right.is_none()
            && root.left_idx == leaf_idx
            && root.right_idx == leaf_idx =>
        {
            Ok(root)
        }
        _ => {
            let mid_idx = root.left_idx + (root.right_idx - root.left_idx) / 2;
            if leaf_idx <= mid_idx {
                find_leaf(
                    root.left
                        .as_ref()
                        .ok_or_else(|| MerkleTreeError::new("invalid left node"))?,
                    leaf_idx,
                )
            } else {
                find_leaf(
                    root.right
                        .as_ref()
                        .ok_or_else(|| MerkleTreeError::new("invalid right node"))?,
                    leaf_idx,
                )
            }
        }
    }
}

// find_parent finds the parent node of the given node.
fn find_parent<'a>(
    root: &'a TreeNode,
    node: &'a TreeNode,
) -> Result<&'a TreeNode, MerkleTreeError> {
    if root == node {
        return Err(MerkleTreeError::new("root node has no parent"));
    }

    // Check if the current root is the parent of the node
    if let Some(left) = &root.left {
        if **left == *node {
            return Ok(root);
        }
    }

    if let Some(right) = &root.right {
        if **right == *node {
            return Ok(root);
        }
    }

    // Recursively search on the left tree
    if let Some(left) = &root.left {
        if let Ok(parent) = find_parent(left, node) {
            return Ok(parent);
        }
    }

    // Recursively search on the right tree
    if let Some(right) = &root.right {
        if let Ok(parent) = find_parent(right, node) {
            return Ok(&parent);
        }
    }

    // If no parent is found, return an error
    Err(MerkleTreeError::new("Parent not found"))
}

// find_sibling finds the sibling node of the given node.
fn find_sibling<'a>(
    root: &'a TreeNode,
    node: &'a TreeNode,
) -> Result<&'a TreeNode, MerkleTreeError> {
    // Find the parent of the node
    let parent = find_parent(root, node)?;

    // Check if the node is the left child
    if let Some(left) = &parent.left {
        if **left == *node {
            return Ok(parent.right.as_ref().unwrap()); // Return the right sibling
        }
    }

    // Check if node is be the right child
    if let Some(right) = &parent.right {
        if **right == *node {
            return Ok(parent.left.as_ref().unwrap()); // Return the left sibling
        }
    }

    // If no sibling is found, return an error
    Err(MerkleTreeError::new("The node has no sibling"))
}

// generate_proof_indices generates proof indices for the leaf node corresponding to the given leaf index.
// It traverses the Merkle tree from the root to the leaf node, collecting the left and right indices
// of each node in the proof path and appends them to the result.
pub fn generate_proof_indices(root: &TreeNode, leaf_idx: usize) -> Result<Vec<[usize; 2]>, MerkleTreeError> {
    let mut result: Vec<[usize; 2]> = Vec::new();

    // Generate the proof nodes
    let nodes = gen_proof(root, leaf_idx)?;

    // Collect the left and right indices of each node in the proof path
    for node in nodes {
        result.push([node.left_idx, node.right_idx]);
    }

    Ok(result)
}
