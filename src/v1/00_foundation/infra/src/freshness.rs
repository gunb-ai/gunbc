//! Mtime-based fast path for freshness checking.
//!
//! In the common case (nothing changed), this avoids reading ~200-400 files
//! by only `stat`-ing them. If any file is newer than the manifest entry, the
//! recorded input file list changed, or the file count changed, it falls
//! through to the full SHA-256 hash path.

use crate::manifest::ManifestEntry;
use std::fmt;
use std::path::{Path, PathBuf};
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
    /// The set of input file paths changed since the manifest entry.
    InputFilesChanged,
    /// The number of input files changed (file added or deleted).
    FileCountChanged { expected: usize, actual: usize },
}

impl fmt::Display for StaleReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StaleReason::FileNewer(path) => {
                write!(f, "file newer than manifest: {}", path.display())
            }
            StaleReason::InputFilesChanged => {
                write!(f, "input file paths changed since manifest")
            }
            StaleReason::FileCountChanged { expected, actual } => {
                write!(
                    f,
                    "file count changed: expected {}, found {}",
                    expected, actual
                )
            }
        }
    }
}

/// File path + mtime (resolved by the caller).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMtime {
    pub path: PathBuf,
    pub modified: SystemTime,
}

/// Check freshness using filesystem mtime as a fast path.
///
/// Returns `Fresh` if all input files are older than `entry.created_at`, the
/// total file count matches `entry.input_file_count`, and any recorded input
/// file list matches exactly. Otherwise returns `MaybeStale` with a reason —
/// the caller should fall through to full hashing.
pub fn check_freshness_mtime(entry: &ManifestEntry, files: &[FileMtime]) -> MtimeResult {
    let created_at = millis_to_system_time(entry.created_at);

    // Check file count
    if files.len() != entry.input_file_count {
        return MtimeResult::MaybeStale(StaleReason::FileCountChanged {
            expected: entry.input_file_count,
            actual: files.len(),
        });
    }

    if let Some(expected_paths) = entry.input_files.as_ref() {
        let input_paths_match = expected_paths.len() == files.len()
            && expected_paths
                .iter()
                .zip(files)
                .all(|(expected, file)| file.path.as_path() == Path::new(expected));
        if !input_paths_match {
            return MtimeResult::MaybeStale(StaleReason::InputFilesChanged);
        }
    }

    for file in files {
        if file.modified > created_at {
            return MtimeResult::MaybeStale(StaleReason::FileNewer(file.path.clone()));
        }
    }

    MtimeResult::Fresh
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

    fn file(path: &str, millis: u64) -> FileMtime {
        FileMtime {
            path: PathBuf::from(path),
            modified: UNIX_EPOCH + Duration::from_millis(millis),
        }
    }

    #[test]
    fn test_fresh_with_old_files() {
        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"), 2)
            .with_input_files(vec!["a.rs".into(), "b.rs".into()])
            .with_timestamp(2_000);
        let files = vec![file("a.rs", 1_000), file("b.rs", 1_500)];

        let result = check_freshness_mtime(&entry, &files);
        assert_eq!(result, MtimeResult::Fresh);
    }

    #[test]
    fn test_stale_with_newer_file() {
        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"), 1).with_timestamp(1_000);
        let files = vec![file("new.rs", 5_000)];

        let result = check_freshness_mtime(&entry, &files);
        assert!(matches!(
            result,
            MtimeResult::MaybeStale(StaleReason::FileNewer(_))
        ));
    }

    #[test]
    fn test_stale_file_count_changed() {
        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"), 3).with_timestamp(2_000);
        let files = vec![file("a.rs", 1_000), file("b.rs", 1_000)];

        let result = check_freshness_mtime(&entry, &files);
        assert!(matches!(
            result,
            MtimeResult::MaybeStale(StaleReason::FileCountChanged {
                expected: 3,
                actual: 2
            })
        ));
    }

    #[test]
    fn test_stale_when_recorded_input_files_change_with_same_count() {
        let entry = ManifestEntry::new(ContentHash::from_bytes(b"test"), 2)
            .with_input_files(vec!["a.rs".into(), "b.rs".into()])
            .with_timestamp(2_000);
        let files = vec![file("a.rs", 1_000), file("c.rs", 1_000)];

        let result = check_freshness_mtime(&entry, &files);
        assert_eq!(
            result,
            MtimeResult::MaybeStale(StaleReason::InputFilesChanged)
        );
    }

    #[test]
    fn test_stale_reason_display() {
        let r = StaleReason::FileNewer(PathBuf::from("foo.rs"));
        assert!(r.to_string().contains("foo.rs"));

        let r = StaleReason::InputFilesChanged;
        assert!(r.to_string().contains("input file"));

        let r = StaleReason::FileCountChanged {
            expected: 5,
            actual: 3,
        };
        assert!(r.to_string().contains("expected 5"));
        assert!(r.to_string().contains("found 3"));
    }
}
