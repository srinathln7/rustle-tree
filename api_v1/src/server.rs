use dotenv::dotenv;
use merkle::MerkleTree;
use std::env;
use std::net::SocketAddr;
use tonic::{transport::Server, Request, Response, Status};

pub mod rustle_tree {
    tonic::include_proto!("rustle_tree");
}

use rustle_tree::{
    merkle_tree_server::{MerkleTree as MerkleTreeTrait, MerkleTreeServer},
    DownloadRequest, DownloadResponse, MerkleProofRequest, MerkleProofResponse, UploadRequest,
    UploadResponse,
};

#[derive(Debug, Default)]
pub struct MerkleTreeService {
    files: Vec<Vec<u8>>,
    merkle_tree: Option<MerkleTree>,
}

#[tonic::async_trait]
impl MerkleTreeTrait for MerkleTreeService {

    async fn upload(&self, request: Request<UploadRequest>) -> Result<Response<UploadResponse>, Status> {
        
        let req = request.into_inner();

        // Build the Merkle tree from the provided files
        let merkle_tree = match merkle::MerkleTree::new(&req.files) {
            Ok(tree) => tree,
            Err(err) => return Err(Status::internal(err.to_string())),
        };

        // Store the files and the Merkle tree
        self.files = req.files;
        self.merkle_tree = Some(merkle_tree);

        // Get the Merkle root
        let merkle_root = match self.merkle_tree.as_ref() {
            Some(tree) => match tree.root {
                Some(root) => root,
                None => return Err(Status::internal("Merkle root not found")),
            },
            None => return Err(Status::internal("Merkle tree not initialized")),
        };


        self.merkle_tree.as_ref().unwrap();

        // Respond with the Merkle root hash
        Ok(Response::new(UploadResponse {
            merkle_root_hash: merkle_root.hash.as_bytes().to_vec(),
        }))
    }




}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let addr: SocketAddr = env::var("SERVER_ADDRESS")
        .unwrap_or_else(|_| "[::1]:50051".to_string())
        .parse()?;
    let merkle_tree_service = MerkleTreeService::default();

    println!("MerkleTreeServer listening on {}", addr);

    Server::builder()
        .add_service(MerkleTreeServer::new(merkle_tree_service))
        .serve(addr)
        .await?;

    Ok(())
}
