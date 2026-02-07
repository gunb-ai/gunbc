//! Emit manifest for tracking emitted artifacts.
//!
//! Records (path, content_hash) pairs for all files emitted by the
//! emit pipeline. Used for CI verification — `make verify-generated`
//! can compare the manifest against actual file contents.

use crate::hash::ContentHash;
use serde::{Deserialize, Serialize};
use std::io;

/// Record of one emitted artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmitRecord {
    pub path: String,
    pub content_hash: ContentHash,
}

/// Manifest of all emitted artifacts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmitManifest {
    pub records: Vec<EmitRecord>,
}

impl EmitManifest {
    /// Parse a manifest from JSON.
    pub fn from_json_str(content: &str) -> io::Result<Self> {
        serde_json::from_str(content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid emit manifest JSON: {}", e),
            )
        })
    }

    /// Serialize this manifest to pretty JSON.
    pub fn to_json_pretty(&self) -> io::Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize emit manifest: {}", e),
            )
        })
    }

    /// Record an emitted artifact.
    pub fn record(&mut self, path: String, hash: ContentHash) {
        self.records.push(EmitRecord {
            path,
            content_hash: hash,
        });
    }

    /// Verify that a path's recorded hash matches the expected hash.
    pub fn verify(&self, path: &str, hash: &ContentHash) -> bool {
        self.records
            .iter()
            .any(|r| r.path == path && &r.content_hash == hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_round_trip() {
        let mut manifest = EmitManifest::default();
        manifest.record(
            "output/Makefile".to_string(),
            ContentHash::from_bytes(b"makefile content"),
        );
        manifest.record(
            "output/ci.yml".to_string(),
            ContentHash::from_bytes(b"ci yaml content"),
        );

        let json = manifest.to_json_pretty().expect("serialize failed");
        let loaded = EmitManifest::from_json_str(&json).expect("parse failed");
        assert_eq!(loaded.records.len(), 2);
        assert_eq!(loaded.records[0].path, "output/Makefile");
        assert_eq!(loaded.records[1].path, "output/ci.yml");
        assert_eq!(loaded.records, manifest.records);
    }

    #[test]
    fn test_manifest_record_and_verify() {
        let mut manifest = EmitManifest::default();
        let hash = ContentHash::from_bytes(b"test content");
        manifest.record("test.rs".to_string(), hash.clone());

        assert!(manifest.verify("test.rs", &hash));
    }

    #[test]
    fn test_manifest_verify_mismatch() {
        let mut manifest = EmitManifest::default();
        let hash = ContentHash::from_bytes(b"test content");
        let wrong_hash = ContentHash::from_bytes(b"different content");
        manifest.record("test.rs".to_string(), hash);

        assert!(!manifest.verify("test.rs", &wrong_hash));
    }

    #[test]
    fn test_manifest_parse_invalid_json() {
        let err = EmitManifest::from_json_str("{not json}").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_manifest_verify_missing_path() {
        let manifest = EmitManifest::default();
        let hash = ContentHash::from_bytes(b"test");
        assert!(!manifest.verify("missing.rs", &hash));
    }
}
