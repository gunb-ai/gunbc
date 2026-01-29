//! Filesystem operations.
//!
//! Operations for working with files: listing, reading, filtering, writing.
//! These can be composed with other flavors to build file-processing DAGs.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_lib_fs::{list_files, filter_by_extension, read_files};
//!
//! let files = list_files("./src")?;
//! let rust_files = filter_by_extension(&files, &["rs"])?;
//! let contents = read_files(&rust_files, "./src")?;
//! ```

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{FileRequest, TransportRequest};
use gunbc_ir::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Filesystem operations for use in DAG nodes.
#[derive(Debug, Clone)]
pub enum FsOp {
    /// List files in a directory (respects .gitignore via git ls-files)
    ListFiles,
    /// Filter files by extension
    FilterByExtension { extensions: Vec<String> },
    /// Read multiple file contents
    ReadFiles,
    /// Read a single file (returns content as string)
    ReadFile,
    /// Prepare a file write request (PURE - no I/O)
    PrepareFileWrite,
}

impl Executable for FsOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            FsOp::ListFiles => {
                let repo_path = inputs
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                
                let files = list_files(repo_path)?;
                
                let mut out = HashMap::new();
                out.insert("files".to_string(), Value::StrList(files));
                Ok(out)
            }
            FsOp::FilterByExtension { extensions } => {
                let files = inputs
                    .get("files")
                    .and_then(|v| v.as_str_list())
                    .ok_or_else(|| ExecError::new("missing or invalid 'files' input"))?;
                
                let ext_strs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
                let filtered = filter_by_extension(&files, &ext_strs);
                
                let mut out = HashMap::new();
                out.insert("files".to_string(), Value::StrList(filtered));
                Ok(out)
            }
            FsOp::ReadFiles => {
                let files = inputs
                    .get("files")
                    .and_then(|v| v.as_str_list())
                    .ok_or_else(|| ExecError::new("missing or invalid 'files' input"))?;
                
                let repo_path = inputs
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                
                let contents = read_files(&files, repo_path)?;
                
                let mut out = HashMap::new();
                out.insert("contents".to_string(), Value::MapStrStr(contents));
                Ok(out)
            }
            FsOp::ReadFile => {
                let path = inputs
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExecError::new("missing or invalid 'path' input"))?;
                
                let content = read_file(path)?;
                
                let mut out = HashMap::new();
                out.insert("content".to_string(), Value::Str(content));
                Ok(out)
            }
            FsOp::PrepareFileWrite => {
                // Accept multiple port names for flexibility, with default
                let path = inputs
                    .get("path")
                    .or_else(|| inputs.get("output_path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("output");  // Default if not provided
                
                let content = inputs
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExecError::new("missing or invalid 'content' input"))?;
                
                let request = prepare_file_write(path, content);
                
                let mut out = HashMap::new();
                out.insert("request".to_string(), Value::Request(request));
                Ok(out)
            }
        }
    }
}

// ============================================================================
// Standalone helper functions - use these like normal library functions
// ============================================================================

/// List files in a directory, respecting .gitignore.
///
/// Uses `git ls-files` when in a git repository, falls back to recursive
/// directory listing otherwise.
///
/// # Example
///
/// ```ignore
/// let files = list_files("./src")?;
/// for file in &files {
///     println!("Found: {}", file);
/// }
/// ```
pub fn list_files(repo_path: &str) -> Result<Vec<String>, ExecError> {
    // Try git ls-files first
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .output()
        .map_err(|e| ExecError::new(format!("failed to run git ls-files: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect();
        return Ok(files);
    }

    // Fallback to recursive listing
    list_files_recursive(Path::new(repo_path))
        .map_err(|e| ExecError::new(format!("failed to list files: {}", e)))
}

/// Filter files by extension.
///
/// # Example
///
/// ```ignore
/// let files = vec!["foo.rs", "bar.py", "baz.rs"];
/// let rust_only = filter_by_extension(&files, &["rs"]);
/// assert_eq!(rust_only, vec!["foo.rs", "baz.rs"]);
/// ```
pub fn filter_by_extension(files: &[String], extensions: &[&str]) -> Vec<String> {
    if extensions.is_empty() {
        return files.to_vec();
    }

    files
        .iter()
        .filter(|f| extensions.iter().any(|ext| f.ends_with(ext)))
        .cloned()
        .collect()
}

/// Read file contents from a list of files.
///
/// Returns a map from filename to contents. Skips files that can't be read
/// (binary, permissions, etc.)
///
/// # Example
///
/// ```ignore
/// let files = vec!["src/lib.rs", "src/main.rs"];
/// let contents = read_files(&files, ".")?;
/// for (name, content) in &contents {
///     println!("{}:\n{}", name, content);
/// }
/// ```
pub fn read_files(files: &[String], repo_path: &str) -> Result<BTreeMap<String, String>, ExecError> {
    let mut contents = BTreeMap::new();

    for file in files {
        let path = Path::new(repo_path).join(file);
        if let Ok(content) = fs::read_to_string(&path) {
            contents.insert(file.clone(), content);
        }
        // Silently skip files that can't be read
    }

    Ok(contents)
}

/// Read a single file's contents.
///
/// # Example
///
/// ```ignore
/// let content = read_file("Cargo.toml")?;
/// println!("{}", content);
/// ```
pub fn read_file(path: &str) -> Result<String, ExecError> {
    fs::read_to_string(path)
        .map_err(|e| ExecError::new(format!("failed to read '{}': {}", path, e)))
}

/// Prepare a file write request (PURE - no I/O).
///
/// Returns a `TransportRequest` that can be executed to write the file.
/// This separates the business logic (deciding what to write) from the
/// actual I/O (writing to disk).
///
/// # Example
///
/// ```ignore
/// let request = prepare_file_write("output.json", &json_content);
/// // Execute with TransportOps::Execute or execute_request
/// ```
pub fn prepare_file_write(path: &str, content: &str) -> TransportRequest {
    TransportRequest::File(FileRequest::write(path, content))
}

// ============================================================================
// Internal helpers
// ============================================================================

fn list_files_recursive(dir: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip hidden directories
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            if path.is_dir() {
                files.extend(list_files_recursive(&path)?);
            } else if let Some(p) = path.to_str() {
                files.push(p.to_string());
            }
        }
    }

    Ok(files)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_extension() {
        let files = vec![
            "foo.rs".to_string(),
            "bar.py".to_string(),
            "baz.rs".to_string(),
            "README.md".to_string(),
        ];

        let result = filter_by_extension(&files, &["rs"]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"foo.rs".to_string()));
        assert!(result.contains(&"baz.rs".to_string()));
    }

    #[test]
    fn test_filter_empty_extensions_returns_all() {
        let files = vec!["foo.rs".to_string(), "bar.py".to_string()];
        let result = filter_by_extension(&files, &[]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_fs_op_filter() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "files".to_string(),
            Value::StrList(vec![
                "foo.rs".to_string(),
                "bar.py".to_string(),
            ]),
        );

        let op = FsOp::FilterByExtension {
            extensions: vec!["rs".to_string()],
        };

        let result = op.execute(inputs).unwrap();
        
        match result.get("files") {
            Some(Value::StrList(files)) => {
                assert_eq!(files.len(), 1);
                assert!(files.contains(&"foo.rs".to_string()));
            }
            _ => panic!("expected file list"),
        }
    }
}
