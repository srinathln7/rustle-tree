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
    // Cpmstructor for Merkle Tree
    pub fn new(files: &[Vec<u8>]) -> Result<MerkleTree, MerkleTreeError> {
        let n = files.len();
        if n == 0 {
            return Err(MerkleTreeError::new("Empty file list"));
        }

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
}
