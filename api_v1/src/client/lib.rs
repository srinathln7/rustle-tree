use dotenv::dotenv;
use rustle_tree::{
    merkle_tree_client::MerkleTreeClient, DownloadRequest, MerkleProofRequest, UploadRequest,
};

use std::env;
use tonic::transport::Channel;
use util::calc_sha256;

// `rustle_tree` refers to the name of the Protobuf package defined in our `.proto` file.
// The tonic crate provides the `include_proto` macro which will generate Rust code from the .proto definitions
// and include it inside the `rustle_tree` module.
pub mod rustle_tree {
    tonic::include_proto!("rustle_tree");
}

#[derive(Debug)]
pub struct UploadResponse {
    pub msg: String,
    pub root_hash: String,
}

#[derive(Debug)]
pub struct DownloadResponse {
    pub msg: String,
    pub file: Vec<u8>,
}

#[derive(Debug)]
pub struct ProofResponse {
    pub msg: String,
    pub proofs: Vec<rustle_tree::TreeNode>,
}

#[derive(Debug)]
pub struct VerifyRequest<'a> {
    pub files: &'a [Vec<u8>],

    pub root_hash: String,
    pub file_idx: usize,
    pub proofs: Vec<rustle_tree::TreeNode>,
}

#[derive(Debug)]
pub struct VerifyResponse {
    pub msg: String,
    pub is_verified: bool,
}

pub async fn setup_grpc_client() -> Result<MerkleTreeClient<Channel>, Box<dyn std::error::Error>> {
    // .ok() suppresses any errors (e.g., if the file doesn't exist).
    dotenv().ok();

    let grpc_server_addr = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set");

    // Ensure the address includes the scheme
    let grpc_server_addr =
        if grpc_server_addr.starts_with("grpc://") || grpc_server_addr.starts_with("grpcs://") {
            grpc_server_addr
        } else {
            format!("grpc://{}", grpc_server_addr)
        };

    println!("gRPC client dialing on server address {}", grpc_server_addr);

    let client = MerkleTreeClient::connect(grpc_server_addr).await?;
    Ok(client)
}

pub async fn upload(
    client: &mut MerkleTreeClient<Channel>,
    files: Vec<Vec<u8>>,
) -> Result<UploadResponse, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(UploadRequest { files });

    // Sends the upload request to the gRPC server. The await keyword ensures that the function suspends and waits for the server's response.
    // In this case since  there are no other asynchronous tasks running concurrently, nothing else happens while waiting for the response.
    // If the server returns an error, the ? operator will propagate the error. `into_inner()`: Extracts the actual response (stripping
    // away the gRPC envelope metadata).
    let response = client.upload(request).await?.into_inner();

    let res = UploadResponse {
        msg: "All files uploaded successfully".to_string(),
        root_hash: String::from_utf8(response.merkle_root_hash).unwrap(),
    };

    println!("Storing the merkle tree root hash on client's disk");
    Ok(res)
}

pub async fn download(
    client: &mut MerkleTreeClient<Channel>,
    file_idx: i64,
) -> Result<DownloadResponse, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(DownloadRequest {
        file_index: file_idx,
    });

    let response = client.download(request).await?.into_inner();

    // format! automatically converts variables (like integers) to strings rather than manual conversion and returns the string for further use
    let msg = format!("file{} downloaded successfully", file_idx);

    Ok(DownloadResponse {
        msg,
        file: response.file_content,
    })
}

pub async fn get_merkle_proof(
    client: &mut MerkleTreeClient<Channel>,
    file_idx: i64,
) -> Result<ProofResponse, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(MerkleProofRequest {
        file_index: file_idx,
    });

    let response = client.get_merkle_proof(request).await?.into_inner();

    let msg = format!("merkle proofs for file{} generated successfully", file_idx);

    Ok(ProofResponse {
        msg,
        proofs: response.proofs,
    })
}

//  The lifetime 'a is used to indicate that the function can borrow data for the duration of the request.
pub async fn verify_merkle_proofs<'a>(
    request: VerifyRequest<'a>,
) -> Result<VerifyResponse, Box<dyn std::error::Error>> {
    // Extract necessary fields from the request
    let VerifyRequest {
        files,

        root_hash,
        file_idx,
        proofs,
    } = request;

    // Calculate the hash of the specified file
    let file_hash = calc_sha256(&files[file_idx as usize]);

    // Convert proofs from Vec<rustle_tree::TreeNode> to Vec<merkle::TreeNode>
    let proof_refs: Vec<merkle::TreeNode> = proofs
        .iter()
        .map(|proof| merkle::TreeNode {
            hash: proof.hash.clone(),
            left_idx: proof.left_idx as usize,
            right_idx: proof.right_idx as usize,
            left: proof.left.as_ref().map(|l| {
                Box::new(merkle::TreeNode {
                    hash: l.hash.clone(),
                    left_idx: l.left_idx as usize,
                    right_idx: l.right_idx as usize,
                    left: None,
                    right: None,
                })
            }),
            right: proof.right.as_ref().map(|r| {
                Box::new(merkle::TreeNode {
                    hash: r.hash.clone(),
                    left_idx: r.left_idx as usize,
                    right_idx: r.right_idx as usize,
                    left: None,
                    right: None,
                })
            }),
        })
        .collect();

    // Create an instance of the Merkle tree (you may need to adjust this based on your implementation)
    let merkle_tree = merkle::MerkleTree::new(files)?;

    // Verify the Merkle proof
    let verification_result = merkle_tree.verify_merkle_proof(
        &root_hash,
        &file_hash,
        file_idx,
        &proof_refs.iter().collect::<Vec<&merkle::TreeNode>>(),
    );

    let is_verified = match verification_result {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error verifying Merkle proof: {}", err);
            return Ok(VerifyResponse {
                msg: format!("Verification failed: {}", err),
                is_verified: false,
            });
        }
    };

    let msg = if is_verified {
        format!("File {} verification successful", file_idx)
    } else {
        format!("File {} verification failed", file_idx)
    };

    Ok(VerifyResponse { msg, is_verified })
}
