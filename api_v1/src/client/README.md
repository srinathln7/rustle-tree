# grpc-client crate Overview

This crate provides functionality for interacting with a gRPC service related to file management and Merkle tree operations. It includes operations for uploading files, downloading files, generating Merkle proofs, and verifying those proofs. 

## `lib.rs` Overview

1. **Imports and Dependencies**
   - **`dotenv`**: Manages environment variables.
   - **`tonic`**: Provides gRPC client capabilities.
   - **`rustle_tree`**: Contains gRPC message definitions for file and Merkle tree operations.
   - **`util::calc_sha256`**: Calculates SHA-256 hashes for file verification.

2. **Module Definitions**
   - **`rustle_tree`**: Includes protocol definitions for gRPC messages.

3. **Data Structures**
   - **`UploadResponse`**: Contains the message and the Merkle tree root hash returned after file upload.
   - **`DownloadResponse`**: Contains the message and the content of the downloaded file.
   - **`ProofResponse`**: Contains the message and Merkle proofs for a file.
   - **`VerifyRequest`**: Encapsulates data needed for verifying Merkle proofs, including files, root hash, file index, and proofs.
   - **`VerifyResponse`**: Contains the result of the Merkle proof verification, including a message and a boolean indicating verification success.

4. **Function Definitions**
   - **`setup_grpc_client`**: Configures and returns a gRPC client connected to the server specified by the `SERVER_ADDRESS` environment variable. Ensures the server address includes the appropriate scheme (`grpc://` or `grpcs://`).
   - **`upload`**: Uploads files to the server and receives the Merkle tree root hash in response.
   - **`download`**: Requests and downloads a file from the server based on its index.
   - **`get_merkle_proof`**: Requests Merkle proofs for a file from the server based on its index.
   - **`verify_merkle_proofs`**: Verifies the Merkle proof for a file by calculating the file hash, converting proof nodes, creating a Merkle tree, and verifying the proof.

## `main.rs` Overview

1. **Initialization**
   - Initializes the gRPC client using `setup_grpc_client`.

2. **File Operations**
   - **Upload**: Uploads a set of files and prints the upload response, including the Merkle tree root hash.
   - **Download**: Downloads a specific file (index 3) and prints its content.
   - **Get Merkle Proof**: Requests Merkle proofs for the file with index 3 and prints the proof response.

3. **Proof Verification**
   - **Positive Case**: Verifies the proof for the file with index 3 using the obtained Merkle root hash and proofs. Prints the verification result.
   - **Negative Case**: Tests verification with modified files and an incorrect root hash. Prints the result, which should indicate a verification failure.

### Summary

This crate functions as a client for a gRPC-based service, facilitating:
- **File Uploads**: Sending files to a server and receiving a Merkle tree root hash.
- **File Downloads**: Retrieving files from the server based on their index.
- **Merkle Proof Generation and Retrieval**: Requesting and retrieving Merkle proofs for files.
- **Merkle Proof Verification**: Independently verifying Merkle proofs using the client's Merkle tree implementation.

The `main.rs` demonstrates practical usage of these functionalities by performing file operations, retrieving proofs, and verifying proofs in both successful and failure scenarios.
