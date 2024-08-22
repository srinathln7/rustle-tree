use sha2::{Digest, Sha256};

pub fn calc_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
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
