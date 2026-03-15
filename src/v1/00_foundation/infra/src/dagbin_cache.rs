//! Content-hash-keyed cache for serialized DAG binaries (`.dagbin` files).
//!
//! Stores compiled `Dag<LoweredOp>` as JSON blobs in `target/dagbin/`, keyed
//! by the source digest of the DSL module that produced them. On cache hit,
//! the runtime skips parse → typecheck → lower → emit entirely, deserializing
//! the cached DAG directly.
//!
//! # Cache structure
//!
//! ```text
//! target/dagbin/
//!   {source_digest_hex}.dagbin     # JSON-serialized Dag<LoweredOp>
//! ```
//!
//! The source digest is computed by `daglang_driver::compute_source_digest_for_context`
//! and incorporates all transitively imported `.dag` files.

use std::io;
use std::path::{Path, PathBuf};

/// Default cache directory relative to the workspace root.
pub const DEFAULT_CACHE_DIR: &str = "target/dagbin";

/// Result of a cache lookup.
#[derive(Debug)]
pub enum CacheLookup {
    /// Cache hit: the raw bytes of the cached `.dagbin` file.
    Hit(Vec<u8>),
    /// Cache miss: no entry for this digest.
    Miss,
}

/// A content-hash-keyed cache for serialized DAG binaries.
///
/// The cache manager is responsible for storing and retrieving serialized
/// `Dag<LoweredOp>` files. It does NOT handle serialization/deserialization
/// itself — callers use `daglang_driver::serialize_lowered_dag` and
/// `daglang_driver::deserialize_lowered_dag` for that.
#[derive(Debug, Clone)]
pub struct DagbinCache {
    /// Root directory for cached `.dagbin` files.
    cache_dir: PathBuf,
}

impl DagbinCache {
    /// Create a cache manager for the given directory.
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
        }
    }

    /// Create a cache manager using the default location under `workspace_root`.
    pub fn from_workspace_root(workspace_root: &Path) -> Self {
        Self::new(workspace_root.join(DEFAULT_CACHE_DIR))
    }

    /// Return the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Compute the `.dagbin` file path for a given source digest.
    pub fn dagbin_path(&self, source_digest: &str) -> PathBuf {
        self.cache_dir.join(format!("{source_digest}.dagbin"))
    }

    /// Look up a cached DAG by source digest.
    ///
    /// Returns `CacheLookup::Hit(bytes)` if a cached file exists and can be read,
    /// `CacheLookup::Miss` if no cached file exists.
    /// Returns `Err` only on I/O errors other than "not found".
    ///
    /// Build-time filesystem access (bootstrap exception).
    #[allow(clippy::disallowed_methods)]
    pub fn load(&self, source_digest: &str) -> io::Result<CacheLookup> {
        let path = self.dagbin_path(source_digest);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(CacheLookup::Hit(bytes)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(CacheLookup::Miss),
            Err(e) => Err(e),
        }
    }

    /// Store serialized DAG bytes under the given source digest.
    ///
    /// Creates the cache directory if it doesn't exist.
    /// Writes atomically via a temporary file to prevent partial reads.
    ///
    /// Build-time filesystem access (bootstrap exception).
    #[allow(clippy::disallowed_methods)]
    pub fn store(&self, source_digest: &str, bytes: &[u8]) -> io::Result<()> {
        std::fs::create_dir_all(&self.cache_dir)?;

        let path = self.dagbin_path(source_digest);

        // Write to a temp file first, then rename for atomicity.
        let tmp_path = path.with_extension("dagbin.tmp");
        std::fs::write(&tmp_path, bytes)?;
        std::fs::rename(&tmp_path, &path)?;

        Ok(())
    }

    /// Check whether a cached entry exists for the given digest without reading it.
    ///
    /// Returns `Ok(true)` when the cache entry exists, `Ok(false)` when it does
    /// not, and propagates other filesystem errors instead of fabricating a miss.
    ///
    /// Build-time filesystem access (bootstrap exception).
    #[allow(clippy::disallowed_methods)]
    pub fn exists(&self, source_digest: &str) -> io::Result<bool> {
        match std::fs::metadata(self.dagbin_path(source_digest)) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Remove a cached entry by source digest.
    ///
    /// Returns `Ok(true)` if a file was removed, `Ok(false)` if it didn't exist.
    ///
    /// Build-time filesystem access (bootstrap exception).
    #[allow(clippy::disallowed_methods)]
    pub fn evict(&self, source_digest: &str) -> io::Result<bool> {
        let path = self.dagbin_path(source_digest);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Remove all cached `.dagbin` files.
    ///
    /// Build-time filesystem access (bootstrap exception).
    #[allow(clippy::disallowed_methods)]
    pub fn clear(&self) -> io::Result<usize> {
        let entries = match std::fs::read_dir(&self.cache_dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e),
        };

        let mut count = 0;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("dagbin") {
                std::fs::remove_file(&path)?;
                count += 1;
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_cache_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("gunbc-cache-dagbin-tests")
            .join(name);
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn store_and_load_round_trip() {
        let dir = temp_cache_dir("round_trip");
        let cache = DagbinCache::new(&dir);
        let digest = "abc123def456";
        let data = b"{\"nodes\":[],\"edges\":[]}";

        cache.store(digest, data).expect("store should succeed");
        let result = cache.load(digest).expect("load should succeed");

        match result {
            CacheLookup::Hit(bytes) => assert_eq!(bytes, data),
            CacheLookup::Miss => panic!("expected cache hit after store"),
        }

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_returns_miss_when_empty() {
        let dir = temp_cache_dir("empty");
        let cache = DagbinCache::new(&dir);

        let result = cache
            .load("nonexistent_digest")
            .expect("load should succeed");
        assert!(matches!(result, CacheLookup::Miss));
    }

    #[test]
    fn exists_reflects_stored_state() {
        let dir = temp_cache_dir("exists");
        let cache = DagbinCache::new(&dir);
        let digest = "exist_check";

        assert!(!cache.exists(digest).expect("exists should succeed"));

        cache.store(digest, b"data").expect("store should succeed");
        assert!(cache.exists(digest).expect("exists should succeed"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn evict_removes_entry() {
        let dir = temp_cache_dir("evict");
        let cache = DagbinCache::new(&dir);
        let digest = "evict_me";

        cache.store(digest, b"data").expect("store should succeed");
        assert!(cache.exists(digest).expect("exists should succeed"));

        let removed = cache.evict(digest).expect("evict should succeed");
        assert!(removed);
        assert!(!cache.exists(digest).expect("exists should succeed"));

        // Evicting again returns false
        let removed = cache.evict(digest).expect("evict should succeed");
        assert!(!removed);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn clear_removes_all_dagbin_files() {
        let dir = temp_cache_dir("clear");
        let cache = DagbinCache::new(&dir);

        cache
            .store("digest1", b"data1")
            .expect("store should succeed");
        cache
            .store("digest2", b"data2")
            .expect("store should succeed");

        let count = cache.clear().expect("clear should succeed");
        assert_eq!(count, 2);
        assert!(!cache.exists("digest1").expect("exists should succeed"));
        assert!(!cache.exists("digest2").expect("exists should succeed"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn clear_on_missing_dir_returns_zero() {
        let dir = temp_cache_dir("clear_missing");
        let cache = DagbinCache::new(&dir);

        let count = cache.clear().expect("clear should succeed");
        assert_eq!(count, 0);
    }

    #[test]
    fn dagbin_path_uses_digest_as_filename() {
        let cache = DagbinCache::new("/tmp/test-dagbin");
        let path = cache.dagbin_path("abcdef0123456789");
        assert_eq!(
            path,
            PathBuf::from("/tmp/test-dagbin/abcdef0123456789.dagbin")
        );
    }

    #[test]
    fn from_workspace_root_uses_default_dir() {
        let cache = DagbinCache::from_workspace_root(Path::new("/workspace"));
        assert_eq!(cache.cache_dir(), Path::new("/workspace/target/dagbin"));
    }

    #[test]
    fn store_overwrites_existing() {
        let dir = temp_cache_dir("overwrite");
        let cache = DagbinCache::new(&dir);
        let digest = "overwrite_me";

        cache
            .store(digest, b"original")
            .expect("store should succeed");
        cache
            .store(digest, b"updated")
            .expect("store should succeed");

        match cache.load(digest).expect("load should succeed") {
            CacheLookup::Hit(bytes) => assert_eq!(bytes, b"updated"),
            CacheLookup::Miss => panic!("expected cache hit"),
        }

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn exists_propagates_filesystem_errors() {
        let dir = temp_cache_dir("exists_error");
        fs::create_dir_all(dir.parent().expect("temp dir should have parent"))
            .expect("parent dir should be created");
        fs::write(&dir, b"not a directory").expect("cache dir placeholder should be written");

        let cache = DagbinCache::new(&dir);
        let result = cache.exists("digest");
        assert!(
            result.is_err(),
            "non-NotFound filesystem errors must propagate"
        );

        fs::remove_file(&dir).ok();
    }
}
