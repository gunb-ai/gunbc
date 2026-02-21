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
