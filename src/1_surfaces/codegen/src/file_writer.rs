//! File writing with dry-run support.

use gunbc_ir::resource::{ResourceError, ResourceIo};
use std::fmt::Write;
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
#[derive(Clone)]
pub struct FileWriter<'a> {
    dry_run: bool,
    io: &'a dyn ResourceIo,
}

impl<'a> FileWriter<'a> {
    /// Create a new file writer.
    pub fn new(dry_run: bool, io: &'a dyn ResourceIo) -> Self {
        Self { dry_run, io }
    }

    /// Create a real-mode file writer.
    pub fn real(io: &'a dyn ResourceIo) -> Self {
        Self::new(false, io)
    }

    /// Create a dry-run file writer.
    pub fn dry_run(io: &'a dyn ResourceIo) -> Self {
        Self::new(true, io)
    }

    /// Check if this writer is in dry-run mode.
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// Write content to a file.
    ///
    /// In dry-run mode, returns what would be written without actually writing.
    pub fn write(
        &self,
        path: impl AsRef<Path>,
        content: impl Into<String>,
    ) -> Result<WriteResult, ResourceError> {
        let path = path.as_ref();
        let content = content.into();
        let path_str = path.display().to_string();

        // Check if content differs from existing file
        let changed = match self.io.read_file(path) {
            Ok(existing) => String::from_utf8(existing)
                .map(|s| s != content)
                .unwrap_or(true),
            Err(_) => true, // File doesn't exist, so it's a change
        };

        if self.dry_run {
            Ok(WriteResult::dry_run(path_str, content, changed))
        } else {
            self.io.write_file(path, content.as_bytes())?;
            Ok(WriteResult::written(path_str, content, changed))
        }
    }

    /// Write content only if it differs from existing file.
    pub fn write_if_changed(
        &self,
        path: impl AsRef<Path>,
        content: impl Into<String>,
    ) -> Result<WriteResult, ResourceError> {
        let path = path.as_ref();
        let content = content.into();
        let path_str = path.display().to_string();

        // Check if content differs from existing file
        let changed = match self.io.read_file(path) {
            Ok(existing) => String::from_utf8(existing)
                .map(|s| s != content)
                .unwrap_or(true),
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
            self.io.write_file(path, content.as_bytes())?;
            Ok(WriteResult::written(path_str, content, true))
        }
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
                writeln!(result, "- {}", old_line).unwrap();
            }
            if !new_line.is_empty() {
                writeln!(result, "+ {}", new_line).unwrap();
            }
        } else {
            writeln!(result, "  {}", old_line).unwrap();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MemoryIo {
        files: std::cell::RefCell<std::collections::HashMap<std::path::PathBuf, Vec<u8>>>,
    }

    impl ResourceIo for MemoryIo {
        fn read_file(&self, path: &Path) -> Result<Vec<u8>, ResourceError> {
            self.files
                .borrow()
                .get(path)
                .cloned()
                .ok_or_else(|| ResourceError::Io(std::io::Error::other("missing file")))
        }

        fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), ResourceError> {
            self.files
                .borrow_mut()
                .insert(path.to_path_buf(), contents.to_vec());
            Ok(())
        }

        fn file_exists(&self, path: &Path) -> Result<bool, ResourceError> {
            Ok(self.files.borrow().contains_key(path))
        }

        fn glob_paths(&self, _pattern: &str) -> Result<Vec<std::path::PathBuf>, ResourceError> {
            Ok(Vec::new())
        }

        fn command_output(
            &self,
            _command: &str,
            _args: &[String],
        ) -> Result<Vec<u8>, ResourceError> {
            Err(ResourceError::Io(std::io::Error::other(
                "command output not supported",
            )))
        }

        fn file_mtime(&self, _path: &Path) -> Result<std::time::SystemTime, ResourceError> {
            Err(ResourceError::Io(std::io::Error::other(
                "mtime not supported",
            )))
        }
    }

    #[test]
    fn test_dry_run_doesnt_write() {
        let io = MemoryIo::default();
        let writer = FileWriter::dry_run(&io);
        let path = std::path::PathBuf::from("dry-run.txt");

        let result = writer.write(&path, "test content").unwrap();

        assert!(!result.written);
        assert!(result.changed);
        assert_eq!(result.content, "test content");
        assert!(!io.file_exists(&path).unwrap());
    }

    #[test]
    fn test_real_mode_writes() {
        let io = MemoryIo::default();
        let writer = FileWriter::real(&io);
        let path = std::path::PathBuf::from("real-write.txt");

        let result = writer.write(&path, "test content").unwrap();

        assert!(result.written);
        assert!(result.changed);

        let content = io.read_file(&path).unwrap();
        assert_eq!(String::from_utf8(content).unwrap(), "test content");
    }

    #[test]
    fn test_write_if_changed_skips_unchanged() {
        let io = MemoryIo::default();
        let writer = FileWriter::real(&io);
        let path = std::path::PathBuf::from("unchanged.txt");

        io.write_file(&path, b"same content").unwrap();

        let result = writer.write_if_changed(&path, "same content").unwrap();

        assert!(!result.written);
        assert!(!result.changed);
    }
}
