# Merkle Crate: Detailed Explanation

This crate implements a Merkle tree structure in Rust, allowing users to build trees, generate proofs, and verify file integrity. Here’s a detailed breakdown of its key components and functionality:

### Error Handling with `MerkleTreeError`
- The `MerkleTreeError` struct represents custom errors that can occur during Merkle tree operations. 
- It stores an error message in a `details` field and implements the `fmt::Display` and `std::error::Error` traits for pretty-printing and error handling.
- A `new` method is provided to easily create an instance of `MerkleTreeError`.

### `TreeNode` Structure
- A `TreeNode` represents a node in the Merkle tree, holding:
  - **hash**: A `String` representing the SHA-256 hash of the node.
  - **left_idx** and **right_idx**: The indices of the files that this node covers in the tree.
  - **left** and **right**: Optional boxed child nodes (`Option<Box<TreeNode>>`), used for internal tree nodes that have children.

- The `TreeNode` struct implements:
  - `Clone`: Allows nodes to be copied, using recursion to clone the entire tree structure.
  - `PartialEq`: Enables equality comparisons between nodes, useful when verifying Merkle proofs.

### `MerkleTree` Structure
- This struct represents the Merkle tree as a whole and holds a root node (`root: Option<Box<TreeNode>>`).
- It also implements the `Clone` trait to allow deep copying of the entire tree.

### Creating a Merkle Tree (`MerkleTree::new`)
- The `new` function constructs a Merkle tree from an array of file data.
  - It returns an error (`MerkleTreeError`) if the file list is empty.
  - If files are provided, it uses the `build_tree` function to recursively build the tree from the bottom up.
  
- The `build_tree` function:
  - Recursively splits the file list into two halves, creating left and right child nodes.
  - Each node's hash is calculated using `calc_sha256`, combining the hashes of its children for internal nodes or hashing the file content for leaf nodes.
  
### Merkle Proof Generation (`generate_merkle_proof`)
- This function generates a Merkle proof for a specific file at `leaf_idx`. 
  - It traverses the tree and collects the sibling nodes needed to verify the file's inclusion in the tree.
  - Proofs are returned as a list of sibling nodes (`Vec<&TreeNode>`).
  - If the leaf index is out of bounds or the root is missing, an error is returned.

### Verifying a Merkle Proof (`verify_merkle_proof`)
- This function verifies the Merkle proof for a file. 
  - It checks if the provided root hash matches the root of the Merkle tree.
  - It also checks whether the file’s hash can be traced to the root of the tree using the proof nodes.
  - If the proof is valid, it returns `true`; otherwise, it returns `false` or an error if any checks fail.

### Helper Functions
- **gen_proof**: Recursively collects sibling nodes to generate the Merkle proof.
- **find_leaf**: Locates the leaf node corresponding to a given file index.
- **find_sibling**: Finds the sibling of a given node.
- **find_parent**: Locates the parent of a node in the tree.

These functions allow traversal of the Merkle tree structure, enabling proof generation and verification.

### Proof Index Generation (`generate_proof_indices`)
- This function generates indices of the nodes involved in the proof path. It’s useful for visualizing or debugging the proof process.
  
### Unit Tests (`mod tests`)
- Tests are provided to validate the correctness of the Merkle tree implementation.
- They test various cases:
  - Empty file lists should result in an error.
  - Correct construction of Merkle trees with different numbers of files.
  - Proper generation and verification of Merkle proofs.

### Summary
This crate provides a flexible and robust implementation of Merkle trees, handling file uploads, proof generation, and verification. It features custom error handling and recursive functions for building and traversing the tree structure. The unit tests ensure that the functionality works as expected across different file scenarios.
