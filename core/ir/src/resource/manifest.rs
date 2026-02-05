//! Resource manifest for tracking freshness state.
//!
//! The manifest stores computed keys for each resource. It's the "upsert key
//! storage" — comparing a resource's computed key to its manifest entry
//! determines whether it's fresh or stale.

use super::super::ResourceId;
use super::hash::ContentHash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Default manifest file location.
pub const DEFAULT_MANIFEST_PATH: &str = "target/.resource-manifest.json";

/// Manifest for tracking resource freshness.
///
/// The manifest is stored on disk and loaded/saved atomically.
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

    /// Files this resource produced (for cleanup/reference).
    #[serde(default)]
    pub outputs: Vec<PathBuf>,
}

impl ManifestEntry {
    /// Create a new manifest entry.
    pub fn new(key: ContentHash) -> Self {
        Self {
            key,
            created_at: current_timestamp_millis(),
            outputs: Vec::new(),
        }
    }

    /// Create an entry with outputs.
    pub fn with_outputs(mut self, outputs: Vec<PathBuf>) -> Self {
        self.outputs = outputs;
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

    /// Load manifest from the default location.
    pub fn load_default() -> io::Result<Self> {
        Self::load(DEFAULT_MANIFEST_PATH)
    }

    /// Load manifest from a file path.
    ///
    /// Returns an empty manifest if the file doesn't exist.
    #[allow(clippy::disallowed_methods)] // Infrastructure code needs direct fs access
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)?;
        let manifest: Self = serde_json::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid manifest JSON: {}", e),
            )
        })?;

        Ok(manifest)
    }

    /// Save manifest to the default location.
    pub fn save_default(&self) -> io::Result<()> {
        self.save(DEFAULT_MANIFEST_PATH)
    }

    /// Save manifest atomically to a file path.
    ///
    /// Uses write-to-temp-then-rename for atomicity.
    #[allow(clippy::disallowed_methods)] // Infrastructure code needs direct fs access
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to temp file
        let tmp_path = path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize manifest: {}", e),
            )
        })?;
        fs::write(&tmp_path, content)?;

        // Atomic rename
        fs::rename(tmp_path, path)?;

        Ok(())
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
#[allow(clippy::disallowed_methods)] // Tests need direct fs access for cleanup
mod tests {
    use super::*;
    use std::env;

    fn temp_manifest_path() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!(
            "test-manifest-{}.json",
            std::process::id()
        ));
        path
    }

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
        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"));

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
        manifest.insert(id.clone(), ManifestEntry::new(key.clone()));

        // Same key — fresh
        assert!(manifest.is_fresh(&id, &key));

        // Different key — not fresh
        let different_key = ContentHash::from_bytes(b"different");
        assert!(!manifest.is_fresh(&id, &different_key));
    }

    #[test]
    fn test_manifest_save_load() {
        let path = temp_manifest_path();

        // Clean up any existing file
        let _ = fs::remove_file(&path);

        // Create and save
        let mut manifest = ResourceManifest::new();
        let id = ResourceId::new("build:test");
        manifest.insert(id.clone(), ManifestEntry::new(ContentHash::from_bytes(b"data")));
        manifest.save(&path).expect("save failed");

        // Load and verify
        let loaded = ResourceManifest::load(&path).expect("load failed");
        assert_eq!(loaded.len(), 1);
        assert!(loaded.get(&id).is_some());

        // Clean up
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_manifest_load_nonexistent() {
        let path = PathBuf::from("/nonexistent/path/manifest.json");
        let manifest = ResourceManifest::load(&path).expect("should return empty");
        assert!(manifest.is_empty());
    }

    #[test]
    fn test_manifest_entry_with_outputs() {
        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"))
            .with_outputs(vec![
                PathBuf::from("output1.txt"),
                PathBuf::from("output2.txt"),
            ]);

        assert_eq!(entry.outputs.len(), 2);
    }

    #[test]
    fn test_manifest_remove() {
        let mut manifest = ResourceManifest::new();
        let id = ResourceId::new("build:test");
        manifest.insert(id.clone(), ManifestEntry::new(ContentHash::from_bytes(b"test")));

        assert_eq!(manifest.len(), 1);
        manifest.remove(&id);
        assert_eq!(manifest.len(), 0);
    }

    #[test]
    fn test_manifest_iter() {
        let mut manifest = ResourceManifest::new();
        manifest.insert(
            ResourceId::new("a"),
            ManifestEntry::new(ContentHash::from_bytes(b"a")),
        );
        manifest.insert(
            ResourceId::new("b"),
            ManifestEntry::new(ContentHash::from_bytes(b"b")),
        );

        let ids: Vec<_> = manifest.iter().map(|(id, _)| id.0.clone()).collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"a".to_string()));
        assert!(ids.contains(&"b".to_string()));
    }
}
