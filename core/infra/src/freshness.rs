//! Mtime-based fast path for freshness checking.
//!
//! In the common case (nothing changed), this avoids reading ~200-400 files
//! by only `stat`-ing them. If any file is newer than the manifest entry or
//! the file count changed, it falls through to the full SHA-256 hash path.

use crate::manifest::ManifestEntry;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Result of an mtime-based freshness check.
#[derive(Debug, PartialEq, Eq)]
pub enum MtimeResult {
    /// All input files are older than the manifest entry and the count matches.
    /// Safe to skip the full hash.
    Fresh,
    /// At least one condition failed — fall through to full hash.
    MaybeStale(StaleReason),
}

/// Why the mtime fast path couldn't confirm freshness.
#[derive(Debug, PartialEq, Eq)]
pub enum StaleReason {
    /// A file has a newer mtime than the manifest entry.
    FileNewer(PathBuf),
    /// The number of input files changed (file added or deleted).
    FileCountChanged { expected: usize, actual: usize },
    /// Could not stat a file.
    StatError(PathBuf, String),
    /// Glob pattern expansion failed.
    GlobError(String),
}

impl fmt::Display for StaleReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StaleReason::FileNewer(path) => write!(f, "file newer than manifest: {}", path.display()),
            StaleReason::FileCountChanged { expected, actual } => {
                write!(f, "file count changed: expected {}, found {}", expected, actual)
            }
            StaleReason::StatError(path, err) => {
                write!(f, "could not stat {}: {}", path.display(), err)
            }
            StaleReason::GlobError(err) => write!(f, "glob error: {}", err),
        }
    }
}

/// Check freshness using filesystem mtime as a fast path.
///
/// Returns `Fresh` if all input files are older than `entry.created_at` and the
/// total file count matches `entry.input_file_count`. Otherwise returns
/// `MaybeStale` with a reason — the caller should fall through to full hashing.
///
/// `glob_patterns`: patterns like `"core/codegen/src/**/*.rs"`
/// `extra_files`: individual files like `"core/codegen/Cargo.toml"`
pub fn check_freshness_mtime(
    entry: &ManifestEntry,
    glob_patterns: &[&str],
    extra_files: &[&str],
) -> MtimeResult {
    let created_at = millis_to_system_time(entry.created_at);
    let mut file_count: usize = 0;

    // Check glob patterns
    for pattern in glob_patterns {
        let entries = match glob::glob(pattern) {
            Ok(entries) => entries,
            Err(e) => {
                return MtimeResult::MaybeStale(StaleReason::GlobError(e.to_string()));
            }
        };

        for path_result in entries {
            let path = match path_result {
                Ok(p) => p,
                Err(e) => {
                    return MtimeResult::MaybeStale(StaleReason::GlobError(e.to_string()));
                }
            };

            if let Some(reason) = check_file_mtime(&path, &created_at) {
                return MtimeResult::MaybeStale(reason);
            }
            file_count += 1;
        }
    }

    // Check extra individual files
    for &path_str in extra_files {
        let path = PathBuf::from(path_str);
        if let Some(reason) = check_file_mtime(&path, &created_at) {
            return MtimeResult::MaybeStale(reason);
        }
        file_count += 1;
    }

    // Check file count
    if file_count != entry.input_file_count {
        return MtimeResult::MaybeStale(StaleReason::FileCountChanged {
            expected: entry.input_file_count,
            actual: file_count,
        });
    }

    MtimeResult::Fresh
}

/// Check a single file's mtime against the threshold.
/// Returns `Some(reason)` if stale, `None` if fresh.
fn check_file_mtime(path: &PathBuf, created_at: &SystemTime) -> Option<StaleReason> {
    let metadata = match stat_file(path) {
        Ok(m) => m,
        Err(e) => {
            return Some(StaleReason::StatError(path.clone(), e.to_string()));
        }
    };

    let mtime = match metadata.modified() {
        Ok(t) => t,
        Err(e) => {
            return Some(StaleReason::StatError(path.clone(), e.to_string()));
        }
    };

    if mtime > *created_at {
        return Some(StaleReason::FileNewer(path.clone()));
    }

    None
}

/// Stat a file (extracted for testability).
fn stat_file(path: &PathBuf) -> io::Result<std::fs::Metadata> {
    std::fs::metadata(path)
}

/// Convert milliseconds since Unix epoch to SystemTime.
fn millis_to_system_time(millis: i64) -> SystemTime {
    if millis >= 0 {
        UNIX_EPOCH + Duration::from_millis(millis as u64)
    } else {
        UNIX_EPOCH
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::ContentHash;
    use std::fs;
    use std::thread;

    fn temp_dir() -> PathBuf {
        crate::test_utils::temp_dir("freshness")
    }

    #[test]
    fn test_fresh_with_old_files() {
        let dir = temp_dir().join("fresh");
        let _ = fs::create_dir_all(&dir);

        // Create files before the manifest entry
        fs::write(dir.join("a.rs"), "fn a() {}").unwrap();
        fs::write(dir.join("b.rs"), "fn b() {}").unwrap();

        // Small delay so manifest timestamp is after file mtimes
        thread::sleep(Duration::from_millis(50));

        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"), 2);
        let pattern = dir.join("*.rs");
        let pattern_str = pattern.to_string_lossy().to_string();

        let result = check_freshness_mtime(&entry, &[pattern_str.as_str()], &[]);
        assert_eq!(result, MtimeResult::Fresh);

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_stale_with_newer_file() {
        let dir = temp_dir().join("stale");
        let _ = fs::create_dir_all(&dir);

        // Create a manifest entry in the past
        let mut entry = ManifestEntry::new(ContentHash::from_bytes(b"test"), 1);
        entry.created_at = 1000; // way in the past

        // Create a file (will have current mtime, newer than 1970)
        fs::write(dir.join("new.rs"), "fn new() {}").unwrap();

        let pattern = dir.join("*.rs");
        let pattern_str = pattern.to_string_lossy().to_string();

        let result = check_freshness_mtime(&entry, &[pattern_str.as_str()], &[]);
        assert!(matches!(result, MtimeResult::MaybeStale(StaleReason::FileNewer(_))));

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_stale_file_count_changed() {
        let dir = temp_dir().join("count");
        let _ = fs::create_dir_all(&dir);

        fs::write(dir.join("a.rs"), "fn a() {}").unwrap();
        fs::write(dir.join("b.rs"), "fn b() {}").unwrap();

        thread::sleep(Duration::from_millis(50));

        // Entry says 3 files, but only 2 exist
        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"), 3);
        let pattern = dir.join("*.rs");
        let pattern_str = pattern.to_string_lossy().to_string();

        let result = check_freshness_mtime(&entry, &[pattern_str.as_str()], &[]);
        assert!(matches!(
            result,
            MtimeResult::MaybeStale(StaleReason::FileCountChanged { expected: 3, actual: 2 })
        ));

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_extra_files_counted() {
        let dir = temp_dir().join("extra");
        let _ = fs::create_dir_all(&dir);

        fs::write(dir.join("a.rs"), "fn a() {}").unwrap();
        let extra = dir.join("Cargo.toml");
        fs::write(&extra, "[package]").unwrap();

        thread::sleep(Duration::from_millis(50));

        // 1 glob file + 1 extra file = 2
        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"), 2);
        let pattern = dir.join("*.rs");
        let pattern_str = pattern.to_string_lossy().to_string();
        let extra_str = extra.to_string_lossy().to_string();

        let result = check_freshness_mtime(&entry, &[pattern_str.as_str()], &[extra_str.as_str()]);
        assert_eq!(result, MtimeResult::Fresh);

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_stale_reason_display() {
        let r = StaleReason::FileNewer(PathBuf::from("foo.rs"));
        assert!(r.to_string().contains("foo.rs"));

        let r = StaleReason::FileCountChanged { expected: 5, actual: 3 };
        assert!(r.to_string().contains("expected 5"));
        assert!(r.to_string().contains("found 3"));
    }
}
