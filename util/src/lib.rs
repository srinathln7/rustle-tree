use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

pub fn calc_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn read_files_from_dir(dir: &str) -> io::Result<Vec<Vec<u8>>> {
    let mut file_contents = Vec::new();
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let mut file = File::open(&path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        file_contents.push(content);
    }

    Ok(file_contents)
}

pub fn write_file(directory: &str, file_name: &str, content: &str) -> io::Result<()> {
    // Create the directory if it doesn't exist
    fs::create_dir_all(directory)?;

    // Create the full file path by joining the directory path and file name
    let file_path = Path::new(directory).join(file_name);

    // Open the file for writing (creates the file if it doesn't exist)
    let mut file = File::create(file_path)?;

    // Write content to the file
    file.write_all(content.as_bytes())?;

    Ok(())
}

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
