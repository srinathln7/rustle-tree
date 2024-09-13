use clap::Parser;
use grpc_client::{
    download, get_merkle_proof, rustle_tree::TreeNode as RustleTreeNode, setup_grpc_client, upload,
};

use merkle::TreeNode;
use std::fs;
use std::path::PathBuf;
use tokio::runtime::Runtime;
use util::{read_files_from_dir, write_file};

/// Rustle Tree CLI for uploading files, downloading files by index, and getting Merkle proofs.
#[derive(Parser, Debug)]
struct Args {
    #[arg(short ='u', long, action = clap::ArgAction::SetTrue)]
    upload: bool,

    #[arg(short = 'd', long, action = clap::ArgAction::SetTrue)]
    download: bool,

    #[arg(short = 'M', long = "getMerkleProofs", action = clap::ArgAction::SetTrue)]
    get_merkle_proofs: bool,

    #[arg(short = 'b', long, action = clap::ArgAction::SetTrue)]
    build_merkle_tree: bool, // New argument for building Merkle tree

    #[arg(short = 'f', long, value_name = "DIR_PATH")]
    files_dir: Option<PathBuf>,

    #[arg(
        short = 'O',
        long,
        value_name = "MERKLE_ROOT_HASH_PATH",
        requires = "upload"
    )]
    merkle_root_hash_path: Option<PathBuf>,

    #[arg(
        short = 'P',
        long,
        value_name = "MERKLE_TREE_PATH",
        requires = "build_merkle_tree"
    )]
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init();

    let args = Args::parse();

    // Initialize the async runtime
    let rt = Runtime::new()?;
    let mut client = rt.block_on(setup_grpc_client())?;

    if args.upload {
        let files_dir = args.files_dir.expect("Files directory required");
        let files = read_files_from_dir(files_dir.to_str().unwrap())?;

        // Print file names and contents for debugging
        for (index, file) in files.iter().enumerate() {
            println!("Uploading file{}: {:?}", index, file); // Debugging output
        }

        let response = rt.block_on(upload(&mut client, files))?;

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
                // Otherwise treat it as a full file path
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

        // Save the Merkle root hash to the specified path
        if let Some(merkle_tree_path) = args.merkle_tree_path {
            write_file(
                merkle_tree_path.parent().unwrap().to_str().unwrap(),
                merkle_tree_path.file_name().unwrap().to_str().unwrap(),
                &merkle_tree_json,
            )?;
            println!("Merkle tree stored at {:?}", merkle_tree_path);
        }
    }

    Ok(())
}

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
