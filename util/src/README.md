# Util crate

This Rust library crate provides utility functions for hashing, file I/O, and basic comparison operations. Below is a breakdown of what each function does:

1. **`calc_sha256(data: &[u8]) -> String`**:
   - Computes the SHA-256 hash of the input byte array (`data`) and returns the hash as a lowercase hexadecimal string.
   - Uses the `sha2` crate for SHA-256 hashing.

2. **`read_files_from_dir(dir: &str) -> io::Result<Vec<Vec<u8>>>`**:
   - Reads the contents of all files in a specified directory (`dir`), returning a vector of byte vectors (`Vec<Vec<u8>>`) where each inner vector represents the content of a file.
   - It filters out non-files (e.g., directories) and sorts the files by their name before reading.
   - Returns the contents of all files, maintaining the sorted order.

3. **`write_file(directory: &str, file_name: &str, content: &str) -> io::Result<()>`**:
   - Writes a string (`content`) to a file in the specified directory (`directory`) with the given `file_name`.
   - Ensures the directory exists, creating it if necessary.
   - Uses `fs::create_dir_all` to create the directory and `File::create` to write the content.

4. **`min(a: usize, b: usize) -> usize`**:
   - A simple utility function that returns the smaller of two unsigned integers.

5. **`max(a: usize, b: usize) -> usize`**:
   - A simple utility function that returns the larger of two unsigned integers.


