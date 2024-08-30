use dotenv::dotenv;
use rustle_tree::{
    merkle_tree_client::MerkleTreeClient, DownloadRequest, MerkleProofRequest, UploadRequest,
};
use std::env;
use tonic::transport::Channel;

//use util::calc_sha256;

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
pub struct VerifyRequest {
    pub root_hash: Vec<u8>,
    pub file_idx: i64,
    pub file: Vec<u8>,
    pub proofs: Vec<rustle_tree::TreeNode>,
}

#[derive(Debug)]
pub struct VerifyResponse {
    pub msg: String,
    pub is_verified: bool,
}

//use tokio::sync::OnceCell;
// static GRPC_CLIENT: OnceCell<MerkleTreeClient<Channel>> = OnceCell::const_new();

pub async fn setup_grpc_client() -> Result<MerkleTreeClient<Channel>, Box<dyn std::error::Error>> {
    dotenv().ok();
    let grpc_server_addr = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set");

    println!("gRPC client dialing on server address {}", grpc_server_addr);

    let client = MerkleTreeClient::connect(grpc_server_addr).await?;
    Ok(client)
}

pub async fn upload(
    client: &mut MerkleTreeClient<Channel>,
    files: Vec<Vec<u8>>,
) -> Result<UploadResponse, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(UploadRequest { files });

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

// pub async fn verify_merkle_proof(
//     client: &mut MerkleTreeClient<Channel>,
//     req: VerifyRequest,
// ) -> Result<VerifyResponse, Box<dyn std::error::Error>> {
//     let request = tonic::Request::new(VerifyProofRequest {
//         root_hash: req.root_hash,
//         file_index: req.file_idx,
//         file_hash: calc_sha256(&req.file).into(),
//         proofs: req.proofs,
//     });

//     let response = client.verify_merkle_proof(request).await?.into_inner();

//     if !response.is_verified {
//         return Err("Merkle verification failed".into());
//     }

//     let msg = format!("merkle verification for file{} is successful", req.file_idx);

//     Ok(VerifyResponse {
//         msg,
//         is_verified: true,
//     })
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Declare the client as mutable
    let mut client = setup_grpc_client().await?;

    let files = vec![vec![1, 2, 3], vec![4, 5, 6]];

    // Pass the client as mutable reference
    let upload_response = upload(&mut client, files).await?;
    println!("Upload response: {:?}", upload_response);

    // Pass the client as mutable reference
    let download_response = download(&mut client, 0).await?;
    println!("Download response: {:?}", download_response);

    // Pass the client as mutable reference
    let proof_response = get_merkle_proof(&mut client, 0).await?;
    println!("Proof response: {:?}", proof_response);

    // let verify_request = VerifyRequest {
    //     root_hash: upload_response.root_hash.into_bytes(),
    //     file_idx: 0,
    //     file: download_response.file,
    //     proofs: proof_response.proofs,
    // };

    // // Pass the client as mutable reference
    // let verify_response = verify_merkle_proof(&mut client, verify_request).await?;
    // println!("Verify response: {:?}", verify_response);

    Ok(())
}
