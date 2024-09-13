#!/bin/bash

# Define paths
CLI_PATH="/home/client/rust/rustle_tree/target/debug/cli"
UPLOAD_DIR="./sample/upload"
OUTPUT_DIR_MERKLE_ROOT="./sample/merkle_root_hash.txt"
OUTPUT_DIR_MERKLE_TREE="./sample/merkle_tree.txt"
DOWNLOAD_DIR="./sample/download"
PROOF_DIR="./sample/merkle-proofs"

# Step 1: Upload files to the server
$CLI_PATH -u -f "$UPLOAD_DIR" -O "$OUTPUT_DIR_MERKLE_ROOT"

# Step 2: Client independently builds the merkle tree later for verification purposes
$CLI_PATH -b -f "$UPLOAD_DIR" -P "$OUTPUT_DIR_MERKLE_TREE"

# Step 3: Delete the uploaded files from the client's disk
# rm -rf "$UPLOAD_DIR"

# Step 4: Download the file with index 0 from the server
mkdir -p "$DOWNLOAD_DIR"  # Ensure the download directory exists
$CLI_PATH -d -i 0 -o "$DOWNLOAD_DIR/file0.txt"  # Ensure a trailing slash to indicate it's a directory

# Download file with index 1
$CLI_PATH -d -i 1 -o "$DOWNLOAD_DIR/file1.txt"

# Download file with index 2
$CLI_PATH -d -i 2 -o "$DOWNLOAD_DIR/file2.txt"

# Download file with index 3
$CLI_PATH -d -i 3 -o "$DOWNLOAD_DIR/file3.txt"


# Step 5: Extract the Merkle proof for file0 from the server
mkdir -p "$PROOF_DIR"
$CLI_PATH -M -i 0 -o "$PROOF_DIR/file0.txt"  # Pass the directory, CLI will append file name

$CLI_PATH -M -i 1 -o "$PROOF_DIR/file1.txt"  # Pass the directory, CLI will append file name

$CLI_PATH -M -i 2 -o "$PROOF_DIR/file2.txt"  # Pass the directory, CLI will append file name

$CLI_PATH -M -i 3 -o "$PROOF_DIR/file3.txt"  # Pass the directory, CLI will append file name

# Step 6: Client independently verifies the integrity of the file without involving the server
$CLI_PATH -v  -P "$OUTPUT_DIR_MERKLE_TREE" -O "$OUTPUT_DIR_MERKLE_ROOT" -f "$DOWNLOAD_DIR" -i 0 -p "$PROOF_DIR/file0.txt"

$CLI_PATH -v  -P "$OUTPUT_DIR_MERKLE_TREE" -O "$OUTPUT_DIR_MERKLE_ROOT" -f "$DOWNLOAD_DIR" -i 1 -p "$PROOF_DIR/file1.txt"

$CLI_PATH -v  -P "$OUTPUT_DIR_MERKLE_TREE" -O "$OUTPUT_DIR_MERKLE_ROOT" -f "$DOWNLOAD_DIR" -i 2 -p "$PROOF_DIR/file2.txt"


$CLI_PATH -v  -P "$OUTPUT_DIR_MERKLE_TREE" -O "$OUTPUT_DIR_MERKLE_ROOT" -f "$DOWNLOAD_DIR" -i 3 -p "$PROOF_DIR/file0.txt" #negative case
$CLI_PATH -v  -P "$OUTPUT_DIR_MERKLE_TREE" -O "$OUTPUT_DIR_MERKLE_ROOT" -f "$DOWNLOAD_DIR" -i 3 -p "$PROOF_DIR/file3.txt" # positive case



# Output success message
echo "Operations completed successfully."
