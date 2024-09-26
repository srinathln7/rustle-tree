use log::info;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use util::calc_sha256;

#[derive(Debug)]
pub struct MerkleTreeError {
    details: String,
}

// Define a `new` method for error
impl MerkleTreeError {
    fn new(msg: &str) -> MerkleTreeError {
        MerkleTreeError {
            details: (msg.to_string()),
        }
    }
}

// implement Display trait on MerkleTreeError to format the error in a custom-defined way.
// <`_> Lifetime annotation is used to indicate the Formatter has a reference tied to the lifetime of the caller.
impl fmt::Display for MerkleTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MerkleTreeError: {}", self.details) // write macro writes to the formatter instead of std. o/p
    }
}

// implement std::error::Error trait for MerkleTreeError
// to integrate it with broader Rust error handling ecosystem
impl Error for MerkleTreeError {}

// Box pointers are used here to enable recursive types, allowing `TreeNode`` to reference itself. They are heap-allocated
// smart pointers, ensuring that the size of the struct remains finite while allowing flexible recursive structures.
// Without Box, Rust would try to allocate the entire tree on the stack, which is not feasible because stack frames have a fixed size.
// The Box pointer stores the TreeNode on the heap, allowing Rust to handle this recursive structure safely and efficiently.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct TreeNode {
    pub hash: String,
    pub left_idx: usize,
    pub right_idx: usize,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
}

// implement clone trait for TreeNode to allow deep copy
// as_ref() method safely accesses the contents of an Option without taking ownership since we only want to borrow the value to clone it
// map() method applies a function to the contents of an Option if it contains Some, allowing transformations like deep cloning
// double dereferencing is required as Box<TreeNode> is still a smart pointer to the TreeNode
impl Clone for TreeNode {
    fn clone(&self) -> Self {
        TreeNode {
            hash: self.hash.clone(),
            left_idx: self.left_idx,
            right_idx: self.right_idx,
            left: self
                .left
                .as_ref()
                .map(|left_node| Box::new((**left_node).clone())),
            right: self
                .right
                .as_ref()
                .map(|right_node| Box::new((**right_node).clone())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleTree {
    pub root: Option<Box<TreeNode>>,
}

// Unlike the Copy trait, which makes shallow copies, Clone can handle more complex types like heap-allocated data (Box).
// Recursive cloning (of child nodes) using Box::new()
impl Clone for MerkleTree {
    fn clone(&self) -> Self {
        MerkleTree {
            root: self
                .root
                .as_ref()
                .map(|root_node| Box::new((**root_node).clone())),
        }
    }
}

impl MerkleTree {
    // Constructor for Merkle Tree
    pub fn new(files: &[Vec<u8>]) -> Result<MerkleTree, MerkleTreeError> {
        let n = files.len();
        if n == 0 {
            return Err(MerkleTreeError::new("empty file list"));
        }

        info!("creating a new Merkle tree with {} files", files.len());
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
    // The use of as_deref() simplifies the conversion of an Option<Box<TreeNode>> to Option<&TreeNode>,
    // allowing us to work with a borrowed reference instead of an owned value. `as_deref()` works with smart pointers.
    // expect call will unwrap the `Option` and will panic only if the root is `None`.
    // Outputs a Vec because the proof is a sequence of references collected during the proof generation process. The Vec allows the function to
    // create and return a new collection that is owned by the caller, while the references inside the Vec point to data owned by the original MerkleTree.
    pub fn generate_merkle_proof(
        &self,
        leaf_idx: usize,
    ) -> Result<Vec<&TreeNode>, MerkleTreeError> {
        info!(
            "[merkle-tree] starting to generate merkle proof for file index {}",
            leaf_idx
        );
        gen_proof(self.root.as_deref().expect("no root"), leaf_idx)
    }

    // Passes only a borrowed slice of references as `proofs: &[&TreeNode]` since it doesn't need to modify or own the proof data.
    // Slices are more lightweight than vectors and sufficient for the verification task, which only reads the data.
    pub fn verify_merkle_proof(
        &self,
        root_hash: &str,
        file_hash: &str,
        file_idx: usize,
        proofs: &[&TreeNode],
    ) -> Result<bool, MerkleTreeError> {
        info!(
            "[merkle-tree] verifying merkle proof for file index {} with merkle root hash {}",
            file_idx, root_hash
        );

        let root = match &self.root {
            Some(root) => root,
            None => return Err(MerkleTreeError::new("empty root")),
        };

        // Deref Coercion: No need to manually dereference the Box with (**root).
        // Rust applies deref coercion to automatically dereference smart pointers like Box making the code simpler and more readable.
        if root.hash != root_hash {
            return Err(MerkleTreeError::new("merkle root hash mismatch"));
        }

        let mut merkle_hash = file_hash.to_string();
        let leaf = find_leaf(root, file_idx)?;

        if leaf.hash != merkle_hash {
            return Ok(false);
        }

        // If the root has either a left or right child
        if root.left.is_some() || root.right.is_some() {
            // Manually create a new MUTABLE `TreeNode` instance
            let mut curr = TreeNode {
                hash: leaf.hash.clone(),
                left: None,
                right: None,
                left_idx: leaf.left_idx,
                right_idx: leaf.right_idx,
            };

            for proof in proofs {
                if curr.left_idx < proof.left_idx && curr.right_idx < proof.right_idx {
                    merkle_hash =
                        calc_sha256(&[merkle_hash.as_bytes(), proof.hash.as_bytes()].concat());
                } else {
                    merkle_hash =
                        calc_sha256(&[proof.hash.as_bytes(), merkle_hash.as_bytes()].concat());
                }

                // Update the indices in the mutable curr node
                curr.left_idx = usize::min(curr.left_idx, proof.left_idx);
                curr.right_idx = usize::max(curr.right_idx, proof.right_idx);
            }
        }

        Ok(root.hash == merkle_hash && root_hash == merkle_hash)
    }

    // Helper function tobe consumed by other module
    pub fn root_hash(&self) -> String {
        match &self.root {
            Some(root) => root.hash.clone(),
            None => String::new(),
        }
    }
}

// gen_proof generates a Merkle proof for the given leaf index.
fn gen_proof(root: &TreeNode, leaf_idx: usize) -> Result<Vec<&TreeNode>, MerkleTreeError> {
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
// `ok_or_else()` is used to convert Option<&Box<TreeNode>> into Result<&Box<TreeNode>, MerkleTreeError>,
// handling the case where a child node is None by returning an error. The ? operator then either unwraps
// the Ok value or returns the Err early, depending on the result.
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
// Lifetimes ('a) in the function tie the root, node, and the returned reference to the same lifetime.
// They ensure that the returned reference (if any) doesn't outlive the input references thereby prevent dangling references (ptrs to data that no longer exists).
// Without lifetimes, Rust's borrow checker would not know how the lifetimes of these references relate, leading to potential memory safety issues.
fn find_parent<'a>(
    root: &'a TreeNode,
    node: &'a TreeNode,
) -> Result<&'a TreeNode, MerkleTreeError> {
    if root == node {
        return Err(MerkleTreeError::new("root node has no parent"));
    }

    // Check if the current root is the parent of the node

    // Check if the left child of the current root is same as the node passed in
    if let Some(left) = &root.left {
        if **left == *node {
            return Ok(root);
        }
    }

    // Check if the right child of the current root is same as the node passed in
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
// as_ref() converts Option<Box<TreeNode>> into Option<&Box<TreeNode>>. This allows us to borrow the Box without taking ownership of it.
// unwrap() retrieves the &Box<TreeNode> from the Option, assuming it is Some. The Box is then automatically dereferenced to &TreeNode, so
// we can access the TreeNode directly without needing to manually dereference the Box.
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
    Err(MerkleTreeError::new("node has no sibling"))
}

// generate_proof_indices generates proof indices for the leaf node corresponding to the given leaf index.
// It traverses the Merkle tree from the root to the leaf node, collecting the left and right indices
// of each node in the proof path and appends them to the result.
pub fn generate_proof_indices(
    root: &TreeNode,
    leaf_idx: usize,
) -> Result<Vec<[usize; 2]>, MerkleTreeError> {
    let mut result: Vec<[usize; 2]> = Vec::new();

    // Generate the proof nodes
    let nodes = gen_proof(root, leaf_idx)?;

    // Collect the left and right indices of each node in the proof path
    for node in nodes {
        result.push([node.left_idx, node.right_idx]);
    }

    Ok(result)
}

// cfg(test) attribute ensures that the tests module is only included when running tests (i.e., it is ignored in the production build)
#[cfg(test)]
mod tests {
    // imports all from parent module to test module allowing the test function to use strcutus, functions without prefixing them
    use super::*;

    #[test]
    fn merkle_tree() {
        // Vec<Vec<u8>> is necessary because it owns the file contents. Each file is a dynamically created vector (Vec<u8>) that is owned by the Vec<Vec<u8>.
        // Vec<&[u8]> would require borrowing data that already exists somewhere, and in our case, we're generating the data on the fly.
        // We need ownership here, which is why Vec<Vec<u8>> is the appropriate choice.
        let tests = vec![
            ("EmptyFile", vec![]),
            // Represents a single file, which is a byte vector containing the ASCII value of "A". b"A" is a byte string literal,
            // representing the byte sequence for the character "A". The `.to_vec()` method converts this byte string into a Vec<u8>.
            ("SingleFile", vec![b"A".to_vec()]),
            (
                "FourFiles",
                vec![b"A".to_vec(), b"B".to_vec(), b"C".to_vec(), b"D".to_vec()],
            ),
            (
                "FiveFiles",
                vec![
                    b"A".to_vec(),
                    b"B".to_vec(),
                    b"C".to_vec(),
                    b"D".to_vec(),
                    b"E".to_vec(),
                ],
            ),
            // The range expression (b'A'..=b'Z') generates all ASCII characters from A to Z as bytes.
            // map(|c| vec![c]) converts each byte into a Vec<u8>, where each character is stored as a vector containing a single byte.
            // collect() gathers all these vectors into a single Vec<Vec<u8>>, representing 26 files, each containing one character (A to Z).
            ("TwentySixFiles", (b'A'..=b'Z').map(|c| vec![c]).collect()),
        ];

        // Test for Empty file
        let (_, files) = &tests[0];

        // Rust's deref coercion -  Converts &Vec<Vec<u8>> to &[Vec<u8>] implictly. These two types are not the same but can be compatible
        let err = MerkleTree::new(files);
        assert_eq!(
            err.unwrap_err().to_string(),
            "MerkleTreeError: empty file list"
        );

        // Test for Five files
        let (_, files) = &tests[3];
        let result = MerkleTree::new(files);

        match result {
            Ok(merkle_tree) => {
                let mut merkle_proof: Vec<Vec<usize>> = Vec::new();
                for file_idx in 0..files.len() {
                    let merkle_proof_idx =
                        generate_proof_indices(merkle_tree.root.as_deref().unwrap(), file_idx)
                            .unwrap(); // Handle the error appropriately

                    // Convert each [usize; 2] into Vec<usize> and append to merkle_proof
                    for idx_pair in merkle_proof_idx {
                        merkle_proof.push(vec![idx_pair[0], idx_pair[1]]);
                    }
                }

                assert_eq!(
                    merkle_proof,
                    vec![
                        vec![1, 1],
                        vec![2, 2],
                        vec![3, 4],
                        vec![0, 0],
                        vec![2, 2],
                        vec![3, 4],
                        vec![0, 1],
                        vec![3, 4],
                        vec![4, 4],
                        vec![0, 2],
                        vec![3, 3],
                        vec![0, 2],
                    ]
                );
            }
            Err(e) => {
                // Handle the error, e.g., assert that an error was expected
                println!("Failed to create Merkle tree: {}", e);
            }
        }

        // Verification test for non-empty files
        for i in 1..tests.len() {
            let (name, files) = &tests[i];
            println!("Running test case: {}", name);

            let merkle_tree = MerkleTree::new(files).expect("MerkleTreeError: empty file list");

            // Forward verification test
            println!("Test case - Merkle verification forward");
            for (idx, file) in files.iter().enumerate() {
                let merkle_proofs_result = merkle_tree.generate_merkle_proof(idx);

                match merkle_proofs_result {
                    Ok(merkle_proofs) => {
                        let is_verified = merkle_tree
                            .verify_merkle_proof(
                                &merkle_tree.root.as_ref().unwrap().hash,
                                &calc_sha256(file),
                                idx,
                                &merkle_proofs,
                            )
                            .unwrap();

                        assert!(
                            is_verified,
                            "Merkle proof verification failed for test {} at file index {}",
                            name, idx
                        );
                    }
                    Err(e) => {
                        panic!("Failed to generate Merkle proof: {}", e);
                    }
                }
            }

            // Reverse verification test
            println!("Test case - merkle verification reverse");
            for idx in (0..files.len()).rev() {
                let merkle_proofs_result = merkle_tree.generate_merkle_proof(idx);

                match merkle_proofs_result {
                    Ok(merkle_proofs) => {
                        let is_verified = merkle_tree
                            .verify_merkle_proof(
                                &merkle_tree.root.as_ref().unwrap().hash,
                                &calc_sha256(&files[idx]),
                                idx,
                                &merkle_proofs,
                            )
                            .unwrap();

                        assert!(
                            is_verified,
                            "Merkle proof verification failed for test {} at file index {}",
                            name, idx
                        );
                    }
                    Err(e) => {
                        panic!("Failed to generate Merkle proof: {}", e);
                    }
                }
            }
        }
    }
}
