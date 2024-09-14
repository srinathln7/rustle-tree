# Rust gRPC Server for Merkle Tree Operations

This gRPC server handles file uploads, downloads, and Merkle proof generation using the `tonic` framework. Below is a detailed explanation of what the code does:

### Importing Libraries
The code begins by importing necessary libraries. These include:
- **dotenv** for loading environment variables,
- **merkle::MerkleTree** for creating and working with Merkle trees,
- **std::env** for accessing environment variables,
- **std::sync::Arc and Mutex** for safely sharing state between multiple threads, and
- **tonic** for building and running a gRPC server.

### Loading Protobuf Definitions
The `rustle_tree` module is generated from the Protobuf definitions using `tonic::include_proto!`. This module contains all the necessary gRPC service and message definitions for communication, including the service traits and the request/response message types.

### Defining Global State
The `GlobalState` struct is defined to hold two fields:
- **files**: A `Vec` of byte arrays representing the uploaded files,
- **merkle_tree**: An optional `MerkleTree` instance.

This state is shared between all client requests. The `GlobalState` struct implements the `Default` trait to initialize the state with an empty list of files and no Merkle tree.

### MerkleTreeService Struct
This struct implements the `MerkleTreeTrait` defined in the Protobuf file. The `MerkleTreeService` holds a reference to the global state, which is protected by a `Mutex` and shared using an `Arc` (atomic reference counting) to ensure thread safety across requests.

### Upload Method
The `upload` method handles file uploads. It takes a request containing files and builds a Merkle tree from them. Once the Merkle tree is created, it is stored along with the files in the global state. The method calculates the Merkle root hash and responds with this value. If any error occurs during tree construction, the method responds with an internal error.

### Download Method
The `download` method retrieves a file by index from the global state. It checks if the requested index is within the valid range of files. If the file exists, it is returned in the response. If the index is out of bounds, the method responds with a "file not found" error.

### get_merkle_proof Method
The `get_merkle_proof` method generates and returns a Merkle proof for a specific file. It first checks if the file index is valid and whether a Merkle tree has been generated. If so, it generates a Merkle proof for the specified file index, converts the proof into a format compatible with the gRPC response, and sends it to the client. If the tree or index is not found, the method returns an error.

### Main Function
The main function sets up and runs the gRPC server. It begins by loading environment variables using `dotenv`. It then retrieves the server address from an environment variable (or defaults to `localhost:50051`). The global state is initialized, and the `MerkleTreeService` is created with this state. Finally, the server is started with the MerkleTree service added, and it listens for client requests on the specified address.

### Error Handling
Throughout the code, errors are handled using the `Result` type. If an operation (such as building a Merkle tree or retrieving a file) fails, the appropriate gRPC `Status` is returned to the client to signal the error.

### Summary
The code implements a basic gRPC server that:
- Accepts file uploads and builds a Merkle tree,
- Allows clients to download files by index,
- Provides Merkle proofs for uploaded files,
- Manages state using `Arc` and `Mutex` for thread safety.

This setup allows multiple clients to interact with the server concurrently, making it a useful framework for blockchain-based or file integrity applications where Merkle trees are required.
