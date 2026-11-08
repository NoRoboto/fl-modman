//! Content hashes: BLAKE3 for every file, SHA-256 only for the exe identity (design §3). Files
//! are hashed by `adapters::fs`.

pub fn b3_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// A short prefix for human output.
pub fn short(hash: &str) -> &str {
    &hash[..hash.len().min(8)]
}
