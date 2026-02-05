//! Test utilities for filesystem operations.
//!
//! These helpers live in `gunbc-infra` because this crate's `clippy.toml`
//! allows direct `std::fs` operations. Downstream crates can call these
//! without `#[allow(clippy::disallowed_methods)]` pragmas.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Create a temporary directory with a unique, labeled name.
///
/// The directory is created immediately. Call [`cleanup_dir`] when done.
pub fn temp_dir(label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    dir.push(format!(
        "gunbc-test-{}-{}-{}",
        label,
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Write a file, creating parent directories if needed.
pub fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create file parent");
    }
    std::fs::write(path, contents).expect("write file");
}

/// Remove a directory tree. Ignores errors (best-effort cleanup).
pub fn cleanup_dir(path: &Path) {
    let _ = std::fs::remove_dir_all(path);
}
