#!/bin/bash

# Define paths
CLI_PATH="/home/client/rust/rustle_tree/target/debug/cli"
UPLOAD_DIR="./sample/upload"
OUTPUT_DIR="./sample/merkle_root_hash.txt"
DOWNLOAD_DIR="./sample/download"
PROOF_DIR="./sample/merkle-proofs"

# Step 1: Upload files to the server
$CLI_PATH -u -f "$UPLOAD_DIR" -O "$OUTPUT_DIR"

# Step 2: Delete the uploaded files from the client's disk
# rm -rf "$UPLOAD_DIR"

# Step 3: Download the file with index 0 from the server
mkdir -p "$DOWNLOAD_DIR"  # Ensure the download directory exists
$CLI_PATH -d -i 0 -o "$DOWNLOAD_DIR/file0.txt"  # Ensure a trailing slash to indicate it's a directory

# Download file with index 1
$CLI_PATH -d -i 1 -o "$DOWNLOAD_DIR/file1.txt"

# Download file with index 2
$CLI_PATH -d -i 2 -o "$DOWNLOAD_DIR/file2.txt"

# Download file with index 3
$CLI_PATH -d -i 3 -o "$DOWNLOAD_DIR/file3.txt"


# Step 4: Extract the Merkle proof for file0 from the server
# mkdir -p "$PROOF_DIR"
$CLI_PATH -M -i 0 -o "$PROOF_DIR/file0.txt"  # Pass the directory, CLI will append file name

$CLI_PATH -M -i 1 -o "$PROOF_DIR/file1.txt"  # Pass the directory, CLI will append file name

$CLI_PATH -M -i 2 -o "$PROOF_DIR/file2.txt"  # Pass the directory, CLI will append file name

$CLI_PATH -M -i 3 -o "$PROOF_DIR/file3.txt"  # Pass the directory, CLI will append file name

# Output success message
echo "Operations completed successfully."
