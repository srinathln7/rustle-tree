# Rustle Tree CLI

The Rustle Tree CLI is a command-line interface for interacting with the Rustle Tree gRPC server. It provides functionality for uploading files, downloading files by index, retrieving Merkle proofs, building a Merkle tree, and verifying proofs.

## Features
- **File Uploading**: Upload a batch of files and retrieve the Merkle root hash.
- **File Downloading**: Download a specific file by its index.
- **Merkle Proofs**: Retrieve and save Merkle proofs for files by their index.
- **Merkle Tree Building**: Build a Merkle tree from local files and save it as JSON.
- **Proof Verification**: Verify a Merkle proof using the Merkle root hash, file hash, and proof nodes.


## Usage

### Upload Files

Upload files from a directory and retrieve the Merkle root hash.

```bash
./target/debug/cli -u -f <FILES_DIR> -O <MERKLE_ROOT_HASH_PATH>
```

- `-u`: Upload flag.
- `-f <FILES_DIR>`: Directory containing the files to upload.
- `-O <MERKLE_ROOT_HASH_PATH>`: Path to save the Merkle root hash.

Example:
```bash
./target/debug/cli -u -f ./sample/upload -O ./merkle_root_hash.json
```

### Download a File

Download a file by its index from the gRPC server.

```bash
./target/debug/cli -d -i <FILE_INDEX> -o <OUTPUT_PATH>
```

- `-d`: Download flag.
- `-i <FILE_INDEX>`: Index of the file to download.
- `-o <OUTPUT_PATH>`: Path to save the downloaded file.

Example:
```bash
./target/debug/cli -d -i 0 -o ./sample/download/file0.txt
```

### Get Merkle Proofs

Retrieve Merkle proofs for a file by its index.

```bash
./target/debug/cli -M -i <FILE_INDEX> -o <PROOF_OUTPUT_PATH>
```

- `-M`: Get Merkle proofs flag.
- `-i <FILE_INDEX>`: Index of the file for which to retrieve Merkle proofs.
- `-o <PROOF_OUTPUT_PATH>`: Path to save the Merkle proof.

Example:
```bash
./target/debug/cli -M -i 0 -o ./sample/merkle-proofs/file0.txt
```

### Build a Merkle Tree Locally

Build a Merkle tree from a directory of files and save it to disk.

```bash
./target/debug/cli -b -f <FILES_DIR> -P <MERKLE_TREE_PATH>
```

- `-b`: Build Merkle tree flag.
- `-f <FILES_DIR>`: Directory containing the files.
- `-P <MERKLE_TREE_PATH>`: Path to save the generated Merkle tree (in JSON format).

Example:
```bash
./target/debug/cli -b -f ./sample/upload -P ./merkle_tree.json
```

### Verify a Merkle Proof

Verify a Merkle proof using a Merkle root hash, file hash, and proof nodes.

```bash
./target/debug/cli -v -f <FILES_DIR> -i <FILE_INDEX> -P <MERKLE_TREE_PATH> -p <PROOF_PATH> -O <MERKLE_ROOT_HASH_PATH>
```

- `-v`: Verify proof flag.
- `-f <FILES_DIR>`: Directory containing the files.
- `-i <FILE_INDEX>`: Index of the file.
- `-P <MERKLE_TREE_PATH>`: Path to the saved Merkle tree (in JSON format).
- `-p <PROOF_PATH>`: Path to the saved proof file (in JSON format).
- `-O <MERKLE_ROOT_HASH_PATH>`: Path to the saved Merkle root hash.

Example:
```bash
./target/debug/cli -v -P ./merkle_tree.json -O ./merkle_root.json -f ./sample/download -i 0  -p ./sample/merkle-proofs/file0.json 
```



