//! Resource manifest for tracking freshness state.
//!
//! The manifest stores computed keys for each resource. It's the "upsert key
//! storage" — comparing a resource's computed key to its manifest entry
//! determines whether it's fresh or stale.

use crate::hash::ContentHash;
use crate::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

/// Default manifest file location.
pub const DEFAULT_MANIFEST_PATH: &str = "target/.resource-manifest.json";

/// Manifest for tracking resource freshness.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceManifest {
    /// Schema version for forward compatibility.
    pub version: u32,

    /// Resources by ID.
    #[serde(default)]
    pub resources: HashMap<ResourceId, ManifestEntry>,
}

/// An entry in the manifest for a single resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Content hash of inputs when resource was created.
    pub key: ContentHash,

    /// When this entry was created (Unix timestamp milliseconds).
    pub created_at: i64,

    /// Number of input files hashed when this entry was created.
    /// Used by the mtime fast path to detect added/deleted files.
    pub input_file_count: usize,

    /// Files this resource produced (for cleanup/reference).
    #[serde(default)]
    pub outputs: Vec<PathBuf>,

    /// Input file paths that were hashed when this entry was created.
    /// Used for diagnostics and debugging stale resources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_files: Option<Vec<String>>,
}

impl ManifestEntry {
    /// Create a new manifest entry.
    pub fn new(key: ContentHash, input_file_count: usize) -> Self {
        Self {
            key,
            created_at: current_timestamp_millis(),
            input_file_count,
            outputs: Vec::new(),
            input_files: None,
        }
    }

    /// Create an entry with outputs.
    pub fn with_outputs(mut self, outputs: Vec<PathBuf>) -> Self {
        self.outputs = outputs;
        self
    }

    /// Create an entry with recorded input file paths.
    pub fn with_input_files(mut self, files: Vec<String>) -> Self {
        self.input_files = Some(files);
        self
    }

    /// Create an entry with a specific timestamp.
    pub fn with_timestamp(mut self, ts: i64) -> Self {
        self.created_at = ts;
        self
    }
}

impl ResourceManifest {
    /// Current manifest schema version.
    pub const CURRENT_VERSION: u32 = 1;

    /// Create a new empty manifest.
    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            resources: HashMap::new(),
        }
    }

    /// Parse a manifest from JSON.
    pub fn from_json_str(content: &str) -> io::Result<Self> {
        let manifest: Self = serde_json::from_str(content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid manifest JSON: {}", e),
            )
        })?;

        Ok(manifest)
    }

    /// Serialize this manifest to pretty JSON.
    pub fn to_json_pretty(&self) -> io::Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize manifest: {}", e),
            )
        })
    }

    /// Get an entry for a resource.
    pub fn get(&self, id: &ResourceId) -> Option<&ManifestEntry> {
        self.resources.get(id)
    }

    /// Insert or update an entry.
    pub fn insert(&mut self, id: ResourceId, entry: ManifestEntry) {
        self.resources.insert(id, entry);
    }

    /// Remove an entry.
    pub fn remove(&mut self, id: &ResourceId) -> Option<ManifestEntry> {
        self.resources.remove(id)
    }

    /// Check if a resource is fresh (key matches).
    pub fn is_fresh(&self, id: &ResourceId, current_key: &ContentHash) -> bool {
        self.get(id)
            .map(|entry| &entry.key == current_key)
            .unwrap_or(false)
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Check if the manifest is empty.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = (&ResourceId, &ManifestEntry)> {
        self.resources.iter()
    }
}

/// Get current timestamp in milliseconds since Unix epoch.
fn current_timestamp_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_new() {
        let manifest = ResourceManifest::new();
        assert_eq!(manifest.version, ResourceManifest::CURRENT_VERSION);
        assert!(manifest.is_empty());
    }

    #[test]
    fn test_manifest_insert_get() {
        let mut manifest = ResourceManifest::new();
        let id = ResourceId::new("build:test");
        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"), 0);

        manifest.insert(id.clone(), entry.clone());

        assert_eq!(manifest.len(), 1);
        assert_eq!(manifest.get(&id), Some(&entry));
    }

    #[test]
    fn test_manifest_is_fresh() {
        let mut manifest = ResourceManifest::new();
        let id = ResourceId::new("build:test");
        let key = ContentHash::from_bytes(b"test");

        // Not in manifest — not fresh
        assert!(!manifest.is_fresh(&id, &key));

        // Add to manifest
        manifest.insert(id.clone(), ManifestEntry::new(key.clone(), 0));

        // Same key — fresh
        assert!(manifest.is_fresh(&id, &key));

        // Different key — not fresh
        let different_key = ContentHash::from_bytes(b"different");
        assert!(!manifest.is_fresh(&id, &different_key));
    }

    #[test]
    fn test_manifest_round_trip() {
        let mut manifest = ResourceManifest::new();
        let id = ResourceId::new("build:test");
        manifest.insert(
            id.clone(),
            ManifestEntry::new(ContentHash::from_bytes(b"data"), 0),
        );

        let json = manifest.to_json_pretty().expect("serialize failed");
        let loaded = ResourceManifest::from_json_str(&json).expect("parse failed");

        assert_eq!(loaded.len(), 1);
        assert!(loaded.get(&id).is_some());
    }

    #[test]
    fn test_manifest_parse_invalid_json() {
        let err = ResourceManifest::from_json_str("{not json}").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_manifest_entry_with_outputs() {
        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"), 0).with_outputs(vec![
            PathBuf::from("output1.txt"),
            PathBuf::from("output2.txt"),
        ]);

        assert_eq!(entry.outputs.len(), 2);
    }

    #[test]
    fn test_manifest_remove() {
        let mut manifest = ResourceManifest::new();
        let id = ResourceId::new("build:test");
        manifest.insert(
            id.clone(),
            ManifestEntry::new(ContentHash::from_bytes(b"test"), 0),
        );

        assert_eq!(manifest.len(), 1);
        manifest.remove(&id);
        assert_eq!(manifest.len(), 0);
    }

    #[test]
    fn test_manifest_iter() {
        let mut manifest = ResourceManifest::new();
        manifest.insert(
            ResourceId::new("a"),
            ManifestEntry::new(ContentHash::from_bytes(b"a"), 0),
        );
        manifest.insert(
            ResourceId::new("b"),
            ManifestEntry::new(ContentHash::from_bytes(b"b"), 0),
        );

        let ids: Vec<_> = manifest.iter().map(|(id, _)| id.0.clone()).collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"a".to_string()));
        assert!(ids.contains(&"b".to_string()));
    }
}
