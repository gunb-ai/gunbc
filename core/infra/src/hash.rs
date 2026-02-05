//! Content hash computation for resource freshness checking.
//!
//! This module provides the `ContentHash` type used as the freshness key for
//! managed resources. The hash is computed from the resource's declared inputs.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

/// A content hash representing the state of a resource's inputs.
///
/// This is the "upsert key" — if the hash matches, the resource is fresh.
/// If it differs, the resource is stale and needs regeneration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(String);

impl ContentHash {
    /// Create a new content hash from a hex string.
    pub fn new(hex: impl Into<String>) -> Self {
        Self(hex.into())
    }

    /// Create a content hash from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self(hex::encode(hasher.finalize()))
    }

    /// Create a content hash from a file's contents.
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let contents = fs::read(path)?;
        Ok(Self::from_bytes(&contents))
    }

    /// Create a content hash from a file's path (for existence-based keys).
    ///
    /// This is used for tools where the key is "does the binary exist at this path?"
    /// The hash incorporates the path string itself.
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        Self::from_bytes(path.as_ref().to_string_lossy().as_bytes())
    }

    /// Get the hex-encoded hash string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create an empty/zero hash (for testing or sentinel values).
    pub fn empty() -> Self {
        Self("0".repeat(64))
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show first 12 chars for readability
        if self.0.len() >= 12 {
            write!(f, "{}...", &self.0[..12])
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl From<ContentHash> for String {
    fn from(hash: ContentHash) -> String {
        hash.0
    }
}

impl From<&ContentHash> for String {
    fn from(hash: &ContentHash) -> String {
        hash.0.clone()
    }
}

/// A builder for computing content hashes from multiple inputs.
///
/// This follows the pattern from the design doc where hash scope is derived
/// from declared inputs. The builder accumulates inputs and produces a final hash.
#[derive(Debug, Default)]
pub struct HashBuilder {
    hasher: Sha256,
}

impl HashBuilder {
    /// Create a new hash builder.
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    /// Add raw bytes to the hash.
    pub fn update(mut self, data: &[u8]) -> Self {
        self.hasher.update(data);
        self
    }

    /// Add a string to the hash.
    pub fn update_str(self, s: &str) -> Self {
        self.update(s.as_bytes())
    }

    /// Add a file's contents to the hash.
    ///
    /// Includes the file path and content length to prevent boundary collisions.
    /// Format: path_bytes + NUL + length_le64 + contents + NUL
    ///
    /// Returns an error if the file cannot be read.
    pub fn update_file(mut self, path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let contents = fs::read(path)?;

        // Hash: path + NUL + length + contents + NUL
        // This prevents boundary collisions (e.g., A="ab",B="c" vs A="a",B="bc")
        self.hasher.update(path.to_string_lossy().as_bytes());
        self.hasher.update([0u8]); // delimiter
        self.hasher.update((contents.len() as u64).to_le_bytes());
        self.hasher.update(&contents);
        self.hasher.update([0u8]); // delimiter

        Ok(self)
    }

    /// Add multiple files matching a glob pattern to the hash.
    ///
    /// Files are sorted by path for deterministic ordering.
    /// Each file contributes: path + NUL + length + contents + NUL
    /// The glob pattern itself is also hashed to distinguish "no matches" states.
    ///
    /// Returns a tuple of (builder, count of files hashed), or an error.
    pub fn update_glob(mut self, pattern: &str) -> io::Result<(Self, usize)> {
        // Hash the glob pattern itself so "no matches" is a distinct contribution
        self.hasher.update(b"glob:");
        self.hasher.update(pattern.as_bytes());
        self.hasher.update([0u8]);

        let entries: Result<Vec<_>, _> = glob::glob(pattern)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?
            .collect();

        // Propagate glob traversal errors instead of silently ignoring them
        let mut paths: Vec<_> = entries.map_err(|e| {
            io::Error::other(
                format!("glob traversal error: {}", e),
            )
        })?;

        // Sort for deterministic ordering
        paths.sort();

        let count = paths.len();
        for path in paths {
            self = self.update_file(&path)?;
        }

        Ok((self, count))
    }

    /// Finalize and return the computed hash.
    pub fn finalize(self) -> ContentHash {
        ContentHash(hex::encode(self.hasher.finalize()))
    }
}

/// Compute a stable hash from string parts.
///
/// Uses SHA-256 with length-prefix encoding to prevent boundary collisions.
/// Returns the first 16 bytes as hex (32 chars) — suitable for stable IDs
/// where a truncated hash is acceptable.
///
/// This is the canonical implementation — all multi-part hashing in the
/// codebase should delegate here.
pub fn hash_parts(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        // Length-prefix each part to prevent collision attacks.
        // Without this, ["a", "b:c"] and ["a:b", "c"] would both hash
        // to the same bytes "a:b:c" and produce identical hashes.
        let len = part.len() as u64;
        hasher.update(len.to_le_bytes());
        hasher.update(part.as_bytes());
    }
    let result = hasher.finalize();
    // Use first 16 bytes as hex (32 chars)
    hex::encode(&result[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_from_bytes() {
        let hash = ContentHash::from_bytes(b"hello world");
        assert_eq!(hash.as_str().len(), 64); // SHA-256 = 64 hex chars
    }

    #[test]
    fn test_content_hash_deterministic() {
        let hash1 = ContentHash::from_bytes(b"test data");
        let hash2 = ContentHash::from_bytes(b"test data");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_content_hash_different_inputs() {
        let hash1 = ContentHash::from_bytes(b"input a");
        let hash2 = ContentHash::from_bytes(b"input b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_content_hash_display() {
        let hash = ContentHash::from_bytes(b"test");
        let display = format!("{}", hash);
        assert!(display.ends_with("..."));
        assert!(display.len() < 20); // Truncated
    }

    #[test]
    fn test_hash_builder() {
        let hash = HashBuilder::new()
            .update(b"part1")
            .update(b"part2")
            .finalize();

        // Same result as hashing concatenated
        let combined = ContentHash::from_bytes(b"part1part2");
        assert_eq!(hash, combined);
    }

    #[test]
    fn test_hash_builder_strings() {
        let hash = HashBuilder::new()
            .update_str("hello")
            .update_str(" ")
            .update_str("world")
            .finalize();

        let expected = ContentHash::from_bytes(b"hello world");
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_empty_hash() {
        let hash = ContentHash::empty();
        assert_eq!(hash.as_str().len(), 64);
        assert!(hash.as_str().chars().all(|c| c == '0'));
    }

    #[test]
    fn test_hash_parts_deterministic() {
        let hash1 = hash_parts(&["check_id", "issue_key"]);
        let hash2 = hash_parts(&["check_id", "issue_key"]);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32); // 16 bytes as hex
    }

    #[test]
    fn test_hash_parts_different_inputs() {
        let hash1 = hash_parts(&["a", "b"]);
        let hash2 = hash_parts(&["c", "d"]);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_parts_order_matters() {
        let hash1 = hash_parts(&["a", "b"]);
        let hash2 = hash_parts(&["b", "a"]);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_parts_no_delimiter_collision() {
        let hash1 = hash_parts(&["a", "b:c"]);
        let hash2 = hash_parts(&["a:b", "c"]);
        assert_ne!(hash1, hash2);

        let hash3 = hash_parts(&["a", "b", "c"]);
        assert_ne!(hash1, hash3);
        assert_ne!(hash2, hash3);
    }
}
