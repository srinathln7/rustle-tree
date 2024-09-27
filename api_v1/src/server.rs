use dotenv::dotenv;
use merkle::MerkleTree;
use std::env;
use std::sync::Arc;
use std::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

// `rustle_tree` refers to the name of the Protobuf package defined in our `.proto` file.
// The tonic crate provides the `include_proto` macro which will generate Rust code from the .proto definitions
// and include it inside the `rustle_tree` module.
pub mod rustle_tree {
    tonic::include_proto!("rustle_tree");
}

// The `MerkleTree` here refers to the trait generated from the service definition in your .proto file. It corresponds to the service `MerkleTree`  defined
// in the proto file. It is renamed as MerkleTreeTrait using as to avoid name conflicts with other items (e.g., a struct or another implementation named MerkleTree).
// MerkleTreeServer: This is the gRPC server implementation generated by tonic. It wraps an implementation of the MerkleTreeTrait and provides the necessary gRPC server
// logic to handle requests from clients.
use rustle_tree::{
    merkle_tree_server::{MerkleTree as MerkleTreeTrait, MerkleTreeServer},
    DownloadRequest, DownloadResponse, MerkleProofRequest, MerkleProofResponse, UploadRequest,
    UploadResponse,
};

#[derive(Debug)]
struct GlobalState {
    files: Vec<Vec<u8>>,
    merkle_tree: Option<MerkleTree>,
}

// Give default values for the `GlobalState` struct
impl Default for GlobalState {
    fn default() -> Self {
        GlobalState {
            files: Vec::new(),
            merkle_tree: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct MerkleTreeService {
    // For a multi-threaded server: Arc allows multiple threads to share ownership of the `global_state` and ensures that it's safe to access across threads.
    // Since accessing mutable data from multiple threads can lead to race conditions, Mutex is used to lock the data when one thread is modifying it
    // ensuring only one thread can modify the data at a time.
    global_state: Arc<Mutex<GlobalState>>,
}

#[tonic::async_trait]
impl MerkleTreeTrait for MerkleTreeService {
    async fn upload(
        &self,
        request: Request<UploadRequest>,
    ) -> Result<Response<UploadResponse>, Status> {
        let req = request.into_inner();

        // Build the Merkle tree from the provided files
        let merkle_tree = match merkle::MerkleTree::new(&req.files) {
            Ok(tree) => tree,
            Err(err) => return Err(Status::internal(err.to_string())),
        };

        // Store the files and Merkle tree in the global state
        let mut global_state = self.global_state.lock().unwrap();
        global_state.files = req.files;
        global_state.merkle_tree = Some(merkle_tree.clone());

        // Calculate the Merkle root hash
        let merkle_root_hash = merkle_tree.root_hash();

        println!("Uploaded all files successfully to the server");

        // Respond with the Merkle root hash
        Ok(Response::new(UploadResponse {
            merkle_root_hash: merkle_root_hash.into_bytes(),
        }))
    }

    async fn download(
        &self,
        request: Request<DownloadRequest>,
    ) -> Result<Response<DownloadResponse>, Status> {
        let req = request.into_inner();
        let file_index = req.file_index as usize;

        // Retrieve the global state
        let global_state = self.global_state.lock().unwrap();

        // Check if the requested index is within the range of stored files
        if file_index >= global_state.files.len() {
            return Err(Status::not_found("File index out of range"));
        }

        // Retrieve the requested file
        let file_data = global_state.files[file_index].clone();

        println!("Downloaded file successfully from the server");

        // Respond with the requested file
        Ok(Response::new(DownloadResponse {
            file_content: file_data,
        }))
    }

    async fn get_merkle_proof(
        &self,
        request: Request<MerkleProofRequest>,
    ) -> Result<Response<MerkleProofResponse>, Status> {
        let req = request.into_inner();
        let file_index = req.file_index as usize;

        // Retrieve the global state
        let global_state = self.global_state.lock().unwrap();

        // Check if the requested index is within the range of stored files
        if file_index >= global_state.files.len() {
            return Err(Status::not_found("File index out of range"));
        }

        // Ensure the Merkle tree is available
        let merkle_tree = match &global_state.merkle_tree {
            Some(tree) => tree,
            None => return Err(Status::internal("Merkle tree not found")),
        };

        // Generate the Merkle proof for the specified file index
        let merkle_proofs = match merkle::MerkleTree::generate_merkle_proof(merkle_tree, file_index)
        {
            Ok(proofs) => proofs,
            Err(err) => return Err(Status::internal(err.to_string())),
        };

        // Convert Vec<&TreeNode> to Vec<restle_tree::TreeNode>
        let mut owned_proofs: Vec<rustle_tree::TreeNode> = Vec::with_capacity(merkle_proofs.len());

        for proof in merkle_proofs {
            let mut api_proof = rustle_tree::TreeNode {
                hash: proof.hash.clone(), // Assuming hash is of type Vec<u8> or similar
                left_idx: proof.left_idx as i64,
                right_idx: proof.right_idx as i64,
                left: None,
                right: None,
            };

            // If there's a left child, create a TreeNode for it
            if let Some(left) = &proof.left {
                api_proof.left = Some(Box::new(rustle_tree::TreeNode {
                    hash: left.hash.clone(),
                    left_idx: left.left_idx as i64,
                    right_idx: left.right_idx as i64,
                    left: None,
                    right: None,
                }));
            }

            // If there's a right child, create a TreeNode for it
            if let Some(right) = &proof.right {
                api_proof.right = Some(Box::new(rustle_tree::TreeNode {
                    hash: right.hash.clone(),
                    left_idx: right.left_idx as i64,
                    right_idx: right.right_idx as i64,
                    left: None,
                    right: None,
                }));
            }

            owned_proofs.push(api_proof);
        }

        println!("Successfully generated merkle proofs");

        // Respond with the requested proofs
        Ok(Response::new(MerkleProofResponse {
            proofs: owned_proofs,
        }))
    }
}

// Tokio is an event-driven, non-blocking I/O platform for writing asynchronous applications with the Rust programming language.
// With #[tokio::main], we can have an async main function, as the macro manages the runtime setup and allows asynchronous operations inside main.
// This macro helps set up a Runtime without requiring the user to use Runtime or Builder directly.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // .ok() suppresses any errors (e.g., if the file doesn't exist).
    dotenv().ok();

    let addr = env::var("SERVER_ADDRESS")
        .unwrap_or_else(|_| "[::1]:50051".to_string())
        .parse()?;

    println!("gRPC server running on {:?}", addr);

    let global_state = Arc::new(Mutex::new(GlobalState::default()));

    // Cloning the Arc means another reference to the same data is created, INCREMENTING the reference count.
    // No actual data copy (cloning) happens, so performance is maintained while allowing multiple tasks to share the same state.
    let service = MerkleTreeService {
        global_state: global_state.clone(),
    };

    Server::builder()
        .add_service(MerkleTreeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
