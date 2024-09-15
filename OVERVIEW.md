# Overview


## Objective
Imagine a client has a large set of potentially small files {F0, F1, …, Fn} and wants to upload them to a server and then delete its local copies. The client wants, however, to later download an arbitrary file from the server and be convinced that the file is correct and is not corrupted in any way (in transport, tampered with by the server, etc.).

You should implement the client, the server and a Merkle tree to support the above (we expect you to implement the Merkle tree rather than use a library, but you are free to use a library for the underlying hash functions). The client must compute a single Merkle tree root hash and keep it on its disk after uploading the files to the server and deleting its local copies. The client can request the i-th file Fi and a Merkle proof Pi for it from the server. The client uses the proof and compares the resulting root hash with the one it persisted before deleting the files - if they match, file is correct.

You have to use Rust. We would like to see a solution with networking that can be deployed across multiple machines, and as close to production-ready as you have time for. Please describe the short-coming your solution have in a report, and how you would improve on them given more time.

## Project Structure

```
.
├── api_v1
│   ├── build.rs
│   ├── Cargo.toml
│   └── src
│       ├── client
│       │   ├── lib.rs
│       │   ├── main.rs
│       │   └── README.md
│       ├── proto
│       │   └── rustle_tree.proto
│       ├── README.md
│       └── server.rs
├── Cargo.lock
├── Cargo.toml
├── cli
│   ├── Cargo.toml
│   └── src
│       ├── main.rs
│       └── README.md
├── docker-compose.yml
├── Dockerfile.cli
├── Dockerfile.grpc-server
├── merkle
│   ├── Cargo.toml
│   └── src
│       ├── lib.rs
│       └── README.md
├── OVERVIEW.md
├── README.md
├── sample
│   └── upload
│       ├── file0.txt
│       ├── file1.txt
│       ├── file2.txt
│       └── file3.txt
├── test.sh
└── util
    ├── Cargo.toml
    └── src
        ├── lib.rs
        └── README.md

12 directories, 29 files

``` 
