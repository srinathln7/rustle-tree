// clap::Parser is used to simplify command-line argument parsing. When you derive the Parser trait from clap,
// it automatically reads and parses arguments passed from the command line and maps them to fields in your struct.
use clap::Parser;
use grpc_client::{
    download, get_merkle_proof, rustle_tree::TreeNode as RustleTreeNode, setup_grpc_client, upload,
};

use merkle::TreeNode;
use std::fs;
use std::path::PathBuf;
use tokio::runtime::Runtime;
use util::{calc_sha256, read_files_from_dir, write_file};

/// Rustle Tree CLI for uploading files, building merkle trees, downloading files by index, generating and verifying Merkle proofs.
#[derive(Parser, Debug)]
struct Args {
    #[arg(short ='u', long, action = clap::ArgAction::SetTrue)]
    upload: bool,

    #[arg(short = 'd', long, action = clap::ArgAction::SetTrue)]
    download: bool,

    #[arg(short = 'M', long = "getMerkleProofs", action = clap::ArgAction::SetTrue)]
    get_merkle_proofs: bool,

    #[arg(short = 'b', long, action = clap::ArgAction::SetTrue)]
    build_merkle_tree: bool,

    #[arg(short = 'v', long, action = clap::ArgAction::SetTrue)]
    verify_proof: bool,

    // PathBuf: cross-platform owned mutable path
    #[arg(short = 'f', long, value_name = "DIR_PATH")]
    files_dir: Option<PathBuf>,

    #[arg(short = 'O', long, value_name = "MERKLE_ROOT_HASH_PATH")]
    merkle_root_hash_path: Option<PathBuf>,

    #[arg(short = 'P', long, value_name = "MERKLE_TREE_PATH")]
    merkle_tree_path: Option<PathBuf>,

    #[arg(
        short = 'i',
        long,
        value_name = "FILE_INDEX",
        conflicts_with = "upload"
    )]
    file_index: Option<i64>,

    #[arg(
        short = 'o',
        long,
        value_name = "OUTPUT_PATH",
        conflicts_with = "upload"
    )]
    output_path: Option<PathBuf>,

    #[arg(
        short = 'p',
        long,
        value_name = "PROOF_PATH",
        requires = "verify_proof"
    )]
    proof_path: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger to output log messages to the console or other configured output.
    // You can control the log level (e.g., info, debug, error) via an environment variable (RUST_LOG), which
    // helps in debugging and tracking program execution without cluttering the code with unnecessary print statements.
    env_logger::init();

    // Parses cmd line arguments into `Args` struct defined using the clap::Parser
    let args = Args::parse();

    // Initialize the async runtime: Since Rust’s main function cannot be `async` (with exception of #[tokio::main] macro), we need a
    // runtime to manage asynchronous tasks. This line initializes the runtime so it can execute async code later.
    let rt = Runtime::new()?;

    // Run an asynchronous function within the sync main function using `block_on` and waits for its completion. Blocks until the current thread
    // is complete. It's purpose is to write async code in a sync way.
    let mut client = rt.block_on(setup_grpc_client())?;

    if args.upload {
        let files_dir = args.files_dir.expect("Files directory required"); // panic if `files_dir` argument is not provided
        let files = read_files_from_dir(files_dir.to_str().unwrap())?;
        let response = rt.block_on(upload(&mut client, files))?;

        // Execute only if `Some(...)` and not None
        if let Some(merkle_root_hash_path) = args.merkle_root_hash_path {
            write_file(
                merkle_root_hash_path.parent().unwrap().to_str().unwrap(),
                merkle_root_hash_path.file_name().unwrap().to_str().unwrap(),
                &response.root_hash,
            )?;

            println!("Merkle root hash stored at {:?}", merkle_root_hash_path);
        }
    } else if args.download {
        let file_index = args.file_index.expect("File index required");
        println!("Requesting file with index: {}", file_index);
        let response = rt.block_on(download(&mut client, file_index))?;

        if let Some(output_path) = args.output_path {
            let output_path = if output_path.is_dir() {
                // Append file name if output path is a directory
                let file_name = format!("file{}.txt", file_index); // e.g., "file0.txt"
                output_path.join(file_name)
            } else {
                // Otherwise treat it as a full file path - clone() is necessary because PathBuf implements the Clone trait to create a deep copy of the path.
                output_path.clone()
            };

            // Ensure the file gets written properly
            fs::write(&output_path, response.file)?;
            println!("File downloaded and stored at {:?}", output_path);
        }
    } else if args.get_merkle_proofs {
        let file_index = args.file_index.expect("File index required");
        let response = rt.block_on(get_merkle_proof(&mut client, file_index))?;

        if let Some(output_path) = args.output_path {
            let output_path = if output_path.is_dir() {
                // Append proof file name if output path is a directory
                let file_name = format!("proof_file{}.json", file_index); // e.g., "proof_file0.json"
                output_path.join(file_name)
            } else {
                output_path.clone()
            };

            // .iter() creates an iterator over the references to each proof node in response.proofs i.e. allow you to traverse the elements of a
            // collection one by one, without consuming or altering the original collection.
            //.collect::<Vec<_>>() consumes the iterator and collects these references into a vector (Vec<&ProofNode>).
            // Vec<_> indicates that we're collecting the iterator's items into a new vector, where `_` is a placeholder that infers the type automatically
            // based on the iterator's output. The `&` in front passes a reference to this vector (&Vec<&ProofNode>).
            let merkle_proofs =
                convert_to_merkle_tree_nodes(&response.proofs.iter().collect::<Vec<_>>());
            let proofs_str = serde_json::to_string(&merkle_proofs)?;

            write_file(
                output_path.parent().unwrap().to_str().unwrap(),
                output_path.file_name().unwrap().to_str().unwrap(),
                &proofs_str,
            )?;
            println!("Merkle proofs stored at {:?}", output_path);
        }
    } else if args.build_merkle_tree {
        // New build Merkle tree functionality
        let files_dir = args.files_dir.expect("Files directory required");
        let files = read_files_from_dir(files_dir.to_str().unwrap())?;

        // Build the Merkle tree from files
        let merkle_tree = merkle::MerkleTree::new(&files)?;

        // Serialize the entire Merkle tree to JSON
        let merkle_tree_json = serde_json::to_string(&merkle_tree)?;

        // Save the Merkle tree to the specified path
        if let Some(merkle_tree_path) = args.merkle_tree_path {
            write_file(
                merkle_tree_path.parent().unwrap().to_str().unwrap(),
                merkle_tree_path.file_name().unwrap().to_str().unwrap(),
                &merkle_tree_json,
            )?;
            println!("Merkle tree stored at {:?}", merkle_tree_path);
        }
    } else if args.verify_proof {
        // New verify proof functionality
        let merkle_tree_path = args.merkle_tree_path.expect("Merkle tree path required");
        let merkle_root_hash_path = args
            .merkle_root_hash_path
            .expect("Merkle root hash path required");
        let file_dir = args.files_dir.expect("File directory required");
        let file_idx = args.file_index.expect("File index required");
        let proof_path = args.proof_path.expect("Proof path directory required");

        // Read Merkle tree from file and de-serialize it to get the `merkle::MerkleTree` struct
        let merkle_tree_json = fs::read_to_string(merkle_tree_path)?;
        let merkle_tree: merkle::MerkleTree = serde_json::from_str(&merkle_tree_json)?;

        // Read Merkle root hash - `trim()` removes any leading or trailing whitespace that might have been included in the file.
        let root_hash = fs::read_to_string(merkle_root_hash_path)?
            .trim()
            .to_string();

        // Read file hash for the file at the provided index
        let files = read_files_from_dir(file_dir.to_str().unwrap())?;
        let file = &files[file_idx as usize];
        let file_hash = calc_sha256(file);

        // Read Merkle proof from the file and de-serialize to retrive the proof struct
        let proofs_json = fs::read_to_string(proof_path)?;
        let proofs: Vec<merkle::TreeNode> = serde_json::from_str(&proofs_json)?;

        // Call the verify_merkle_proof function
        // Conv. the proofs into a vector of references to TreeNode structs, which is needed for the verification.
        let is_valid = merkle_tree.verify_merkle_proof(
            &root_hash,
            &file_hash,
            file_idx as usize,
            &proofs.iter().collect::<Vec<_>>(),
        )?;

        if is_valid {
            println!("\x1b[32mProof verified successfully.\x1b[0m");
        } else {
            println!("\x1b[31mFailed to verify proof.\x1b[0m");
        }
    }

    Ok(())
}

// iter(): Borrows each element (&T), so the original collection remains unchanged.
fn convert_to_merkle_tree_nodes(nodes: &[&RustleTreeNode]) -> Vec<TreeNode> {
    nodes
        .iter()
        .map(|node| TreeNode {
            hash: node.hash.clone(),
            left_idx: node.left_idx as usize,
            right_idx: node.right_idx as usize,
            left: node
                .left
                .as_ref()
                .map(|left_node| Box::new(convert_to_merkle_tree_node(left_node))),
            right: node
                .right
                .as_ref()
                .map(|right_node| Box::new(convert_to_merkle_tree_node(right_node))),
        })
        .collect()
}

// Recursive conversion of node and its children
fn convert_to_merkle_tree_node(node: &RustleTreeNode) -> TreeNode {
    TreeNode {
        hash: node.hash.clone(),
        left_idx: node.left_idx as usize,
        right_idx: node.right_idx as usize,
        left: node
            .left
            .as_ref()
            .map(|left_node| Box::new(convert_to_merkle_tree_node(left_node))),
        right: node
            .right
            .as_ref()
            .map(|right_node| Box::new(convert_to_merkle_tree_node(right_node))),
    }
}
