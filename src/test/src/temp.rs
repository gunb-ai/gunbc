use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Create a unique temporary directory path based on a namespace label,
/// process ID, and the current time in nanoseconds.
pub fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "gunbc_test_{label}_{}_{}",
        std::process::id(),
        nanos
    ))
}

/// Create a unique temporary `.dag` file path inside a fresh directory.
///
/// The parent directory is created automatically. Returns the path to
/// `<temp_dir>/<label>.dag`.
#[allow(clippy::disallowed_methods)]
pub fn unique_temp_file(label: &str) -> PathBuf {
    let root = unique_temp_dir(label);
    std::fs::create_dir_all(&root).expect("failed to create temp fixture root");
    root.join(format!("{label}.dag"))
}
