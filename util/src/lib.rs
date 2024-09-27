use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

pub fn calc_sha256(data: &[u8]) -> String {
    // A new instance of a hasher is initialized. `update` method feeds the input data into the hasher for processing.
    // The finalize method completes the hashing process and produces the final hash value
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

// Returns a Result containing a vector of byte vectors where the outer vector represents multiple files,
// while each inner vector contains the bytes of a single file.
pub fn read_files_from_dir(dir: &str) -> io::Result<Vec<Vec<u8>>> {
    let mut file_contents = Vec::new();

    // Collect entries and sort by file name
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok()) // Remove any Err variants and keep only the `Ok` variants
        .filter(|e| e.path().is_file()) // Only process files
        .collect(); // collect the filtered entries into the vector

    entries.sort_by_key(|entry| entry.file_name()); // Sort by file name

    for entry in entries {
        // Obtain the path for each file entry
        let path = entry.path();

        let mut file = File::open(&path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?; // entire file content is read into the `content` vector

        // Each file content is pushed into the `file_contents` vector
        file_contents.push(content);
    }

    Ok(file_contents)
}

pub fn write_file(directory: &str, file_name: &str, content: &str) -> io::Result<()> {
    // Create the directory if it doesn't exist
    fs::create_dir_all(directory)?;

    // Create the full file path by joining the directory path and file name
    let file_path = Path::new(directory).join(file_name);

    // Open the file for writing (creates the file if it doesn't exist and opens the file in write-only mode)
    let mut file = File::create(file_path)?;

    // Write content to the file
    file.write_all(content.as_bytes())?;

    Ok(())
}

// Since the `min` and `max` function takes in a value type (`usize`) we don't require lifetime annotations here
// because they own their data and  Rust creates a copy of that value in the function’s stack frame. If we were
// to modify the fucntions to take references, then we would need to specify lifetimes. Lifetimes are primarily
// used to ensure that references (& types) are valid for a certain duration
pub fn min(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}

pub fn max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}
