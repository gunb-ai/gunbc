//! File writing with dry-run support.

use std::fs;
use std::io;
use std::path::Path;

/// Result of a file write operation.
#[derive(Debug)]
pub struct WriteResult {
    /// Path that was (or would be) written
    pub path: String,
    /// Whether the file was actually written
    pub written: bool,
    /// The content that was (or would be) written
    pub content: String,
    /// Whether the content differs from existing file
    pub changed: bool,
}

impl WriteResult {
    /// Create a dry-run result.
    pub fn dry_run(path: impl Into<String>, content: impl Into<String>, changed: bool) -> Self {
        Self {
            path: path.into(),
            written: false,
            content: content.into(),
            changed,
        }
    }

    /// Create a written result.
    pub fn written(path: impl Into<String>, content: impl Into<String>, changed: bool) -> Self {
        Self {
            path: path.into(),
            written: true,
            content: content.into(),
            changed,
        }
    }
}

/// File writer with dry-run support.
#[derive(Debug, Clone)]
pub struct FileWriter {
    dry_run: bool,
}

impl FileWriter {
    /// Create a new file writer.
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// Create a real-mode file writer.
    pub fn real() -> Self {
        Self::new(false)
    }

    /// Create a dry-run file writer.
    pub fn dry_run() -> Self {
        Self::new(true)
    }

    /// Check if this writer is in dry-run mode.
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// Write content to a file.
    ///
    /// In dry-run mode, returns what would be written without actually writing.
    pub fn write(&self, path: impl AsRef<Path>, content: impl Into<String>) -> io::Result<WriteResult> {
        let path = path.as_ref();
        let content = content.into();
        let path_str = path.display().to_string();

        // Check if content differs from existing file
        let changed = match fs::read_to_string(path) {
            Ok(existing) => existing != content,
            Err(_) => true, // File doesn't exist, so it's a change
        };

        if self.dry_run {
            Ok(WriteResult::dry_run(path_str, content, changed))
        } else {
            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, &content)?;
            Ok(WriteResult::written(path_str, content, changed))
        }
    }

    /// Write content only if it differs from existing file.
    pub fn write_if_changed(&self, path: impl AsRef<Path>, content: impl Into<String>) -> io::Result<WriteResult> {
        let path = path.as_ref();
        let content = content.into();
        let path_str = path.display().to_string();

        // Check if content differs from existing file
        let changed = match fs::read_to_string(path) {
            Ok(existing) => existing != content,
            Err(_) => true,
        };

        if !changed {
            return Ok(WriteResult {
                path: path_str,
                written: false,
                content,
                changed: false,
            });
        }

        if self.dry_run {
            Ok(WriteResult::dry_run(path_str, content, true))
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, &content)?;
            Ok(WriteResult::written(path_str, content, true))
        }
    }
}

impl Default for FileWriter {
    fn default() -> Self {
        Self::real()
    }
}

/// Format a diff between two strings.
pub fn format_diff(old: &str, new: &str) -> String {
    let mut result = String::new();
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // Simple line-by-line diff
    let max_len = old_lines.len().max(new_lines.len());
    for i in 0..max_len {
        let old_line = old_lines.get(i).copied().unwrap_or("");
        let new_line = new_lines.get(i).copied().unwrap_or("");

        if old_line != new_line {
            if !old_line.is_empty() {
                result.push_str(&format!("- {}\n", old_line));
            }
            if !new_line.is_empty() {
                result.push_str(&format!("+ {}\n", new_line));
            }
        } else {
            result.push_str(&format!("  {}\n", old_line));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_dry_run_doesnt_write() {
        let writer = FileWriter::dry_run();
        let temp_path = env::temp_dir().join("gunbc-test-dry-run.txt");

        // Clean up any existing file
        let _ = fs::remove_file(&temp_path);

        let result = writer.write(&temp_path, "test content").unwrap();

        assert!(!result.written);
        assert!(result.changed);
        assert_eq!(result.content, "test content");

        // File should not exist
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_real_mode_writes() {
        let writer = FileWriter::real();
        let temp_path = env::temp_dir().join("gunbc-test-real-write.txt");

        // Clean up any existing file
        let _ = fs::remove_file(&temp_path);

        let result = writer.write(&temp_path, "test content").unwrap();

        assert!(result.written);
        assert!(result.changed);

        // File should exist with correct content
        let content = fs::read_to_string(&temp_path).unwrap();
        assert_eq!(content, "test content");

        // Clean up
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_write_if_changed_skips_unchanged() {
        let writer = FileWriter::real();
        let temp_path = env::temp_dir().join("gunbc-test-unchanged.txt");

        // Write initial content
        fs::write(&temp_path, "same content").unwrap();

        let result = writer.write_if_changed(&temp_path, "same content").unwrap();

        assert!(!result.written);
        assert!(!result.changed);

        // Clean up
        let _ = fs::remove_file(&temp_path);
    }
}
