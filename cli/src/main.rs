use clap::Parser;
use grpc_client::{download, get_merkle_proof, setup_grpc_client, upload};
use std::fs;
use std::path::PathBuf;
use tokio::runtime::Runtime;
use util::{read_files_from_dir, write_file};

/// Rustle Tree CLI for uploading files, downloading files by index, and getting Merkle proofs.
#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    upload: bool,

    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    download: bool,

    #[arg(short = 'M', long = "getMerkleProofs", action = clap::ArgAction::SetTrue)]
    get_merkle_proofs: bool,

    #[arg(short = 'd', long, value_name = "DIR_PATH", requires = "upload")]
    files_dir: Option<PathBuf>,

    #[arg(
        short = 'O',
        long,
        value_name = "MERKLE_ROOT_HASH_PATH",
        requires = "upload"
    )]
    merkle_root_hash_path: Option<PathBuf>,

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
        let response = rt.block_on(download(&mut client, file_index))?;

        if let Some(output_path) = args.output_path {
            fs::write(output_path.clone(), response.file)?;
            println!("File downloaded and stored at {:?}", output_path);
        }
    } else if args.get_merkle_proofs {
        let file_index = args.file_index.expect("File index required");
        let response = rt.block_on(get_merkle_proof(&mut client, file_index))?;

        if let Some(output_path) = args.output_path {
            let proofs_str = serde_json::to_string(&response.proofs)?;
            write_file(
                output_path.parent().unwrap().to_str().unwrap(),
                output_path.file_name().unwrap().to_str().unwrap(),
                &proofs_str,
            )?;
            println!("Merkle proofs stored at {:?}", output_path);
        }
    }

    Ok(())
}
