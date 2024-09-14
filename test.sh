#!/bin/bash

# Define paths
CLI_PATH="./target/release/cli"
UPLOAD_DIR="./sample/upload"
OUTPUT_DIR_MERKLE_ROOT="./sample/merkle_root_hash.json"
OUTPUT_DIR_MERKLE_TREE="./sample/merkle_tree.json"
DOWNLOAD_DIR="./sample/download"
PROOF_DIR="./sample/merkle-proofs"

# Define color codes
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Step 1: Client independently builds the merkle tree later for verification purposes
echo -e "${YELLOW}Client independently computing the merkle tree and the root hash${NC}"
$CLI_PATH -b -f "$UPLOAD_DIR" -P "$OUTPUT_DIR_MERKLE_TREE"

# Step 2: Upload files to the server
echo -e "${YELLOW}Uploading files to the server${NC}"
$CLI_PATH -u -f "$UPLOAD_DIR" -O "$OUTPUT_DIR_MERKLE_ROOT"

# Step 3: Delete the uploaded files from the client's disk
echo -e "${YELLOW}Client erasing all the uploaded files from the disk${NC}"
rm -rf "$UPLOAD_DIR"

# Step 4: Download the file with index 0 from the server
mkdir -p "$DOWNLOAD_DIR"  # Ensure the download directory exists

echo -e "${YELLOW}Client downloading file0 from the grpc-server${NC}"
$CLI_PATH -d -i 0 -o "$DOWNLOAD_DIR/file0.txt"  # Ensure a trailing slash to indicate it's a directory

# Download file with index 1
echo -e "${YELLOW}Client downloading file1 from the grpc-server${NC}"
$CLI_PATH -d -i 1 -o "$DOWNLOAD_DIR/file1.txt"

# Download file with index 2
echo -e "${YELLOW}Client downloading file2 from the grpc-server${NC}"
$CLI_PATH -d -i 2 -o "$DOWNLOAD_DIR/file2.txt"

# Download file with index 3
echo -e "${YELLOW}Client downloading file3 from the grpc-server${NC}"
$CLI_PATH -d -i 3 -o "$DOWNLOAD_DIR/file3.txt"

# Step 5: Extract the Merkle proof for file0 from the server
mkdir -p "$PROOF_DIR"

echo -e "${YELLOW}Client requesting merkle proofs for file0 from the grpc-server${NC}"
$CLI_PATH -M -i 0 -o "$PROOF_DIR/file0.json"  # Pass the directory, CLI will append file name

echo -e "${YELLOW}Client requesting merkle proofs for file1 from the grpc-server${NC}"
$CLI_PATH -M -i 1 -o "$PROOF_DIR/file1.json"  # Pass the directory, CLI will append file name

echo -e "${YELLOW}Client requesting merkle proofs for file2 from the grpc-server${NC}"
$CLI_PATH -M -i 2 -o "$PROOF_DIR/file2.json"  # Pass the directory, CLI will append file name

echo -e "${YELLOW}Client requesting merkle proofs for file3 from the grpc-server${NC}"
$CLI_PATH -M -i 3 -o "$PROOF_DIR/file3.json"  # Pass the directory, CLI will append file name

# Step 6: Client independently verifies the integrity of the file without involving the server

echo -e "${YELLOW}Client independently verifying merkle proofs for file0 - positive case${NC}"
$CLI_PATH -v  -P "$OUTPUT_DIR_MERKLE_TREE" -O "$OUTPUT_DIR_MERKLE_ROOT" -f "$DOWNLOAD_DIR" -i 0 -p "$PROOF_DIR/file0.json"

echo -e "${YELLOW}Client independently verifying merkle proofs for file1 - positive case${NC}"
$CLI_PATH -v  -P "$OUTPUT_DIR_MERKLE_TREE" -O "$OUTPUT_DIR_MERKLE_ROOT" -f "$DOWNLOAD_DIR" -i 1 -p "$PROOF_DIR/file1.json"

echo -e "${YELLOW}Client independently verifying merkle proofs for file2 - positive case${NC}"
$CLI_PATH -v  -P "$OUTPUT_DIR_MERKLE_TREE" -O "$OUTPUT_DIR_MERKLE_ROOT" -f "$DOWNLOAD_DIR" -i 2 -p "$PROOF_DIR/file2.json"

echo -e "${YELLOW}Client independently verifying merkle proofs for file3 - negative case${NC}"
$CLI_PATH -v  -P "$OUTPUT_DIR_MERKLE_TREE" -O "$OUTPUT_DIR_MERKLE_ROOT" -f "$DOWNLOAD_DIR" -i 3 -p "$PROOF_DIR/file0.json" #negative case

echo -e "${YELLOW}Client independently verifying merkle proofs for file3 - positive case${NC}"
$CLI_PATH -v  -P "$OUTPUT_DIR_MERKLE_TREE" -O "$OUTPUT_DIR_MERKLE_ROOT" -f "$DOWNLOAD_DIR" -i 3 -p "$PROOF_DIR/file3.json" # positive case

# Output success message
echo -e "${GREEN}All operations completed successfully.${NC}"
