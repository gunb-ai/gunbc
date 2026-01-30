//! I/O primitives - world interactions (boundaries).
//!
//! ## Architecture Note
//!
//! This module follows the **transport pattern**: pure "Prepare" ops produce
//! `TransportRequest` values, which are then executed by `TransportOps::Execute`.
//! This separation enables:
//! - Centralized I/O interception for dry-run mode
//! - Consistent mocking/testing
//! - Policy enforcement at the transport layer
//!
//! **Preferred pattern:**
//! - Use `PrepareFileReadOp`, `PrepareFileWriteOp`, `PrepareShellOp` (pure)
//! - Execute via `TransportOps::Execute` (single I/O boundary)
//!
//! **Deprecated (direct I/O):**
//! - `ReadFileOp`, `WriteFileOp`, `ExecuteOp` perform I/O directly and bypass
//!   the transport layer. These are deprecated and will be removed in a future version.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{FileRequest, HttpMethod, RestRequest, ShellRequest, TransportRequest};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

// =============================================================================
// DEPRECATED: Direct I/O Operations (bypass transport layer)
// =============================================================================

/// Read a file from the filesystem.
///
/// **DEPRECATED**: Use `PrepareFileReadOp` + `TransportOps::Execute` instead.
/// This op performs I/O directly, bypassing the transport layer.
///
/// Inputs:
/// - `path`: String path to the file
///
/// Outputs:
/// - `content`: String contents of the file
/// - `exists`: Bool indicating if the file exists
#[deprecated(
    since = "0.2.0",
    note = "Use PrepareFileReadOp + TransportOps::Execute instead"
)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadFileOp;

impl Executable for ReadFileOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let path = inputs
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'path' string"))?;

        let mut out = HashMap::new();

        if Path::new(path).exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| ExecError::new(format!("failed to read file '{}': {}", path, e)))?;
            out.insert("content".to_string(), Value::Str(content));
            out.insert("exists".to_string(), Value::Bool(true));
        } else {
            out.insert("content".to_string(), Value::Str(String::new()));
            out.insert("exists".to_string(), Value::Bool(false));
        }

        Ok(out)
    }
}

/// Write a file to the filesystem (boundary operation).
///
/// **DEPRECATED**: Use `PrepareFileWriteOp` + `TransportOps::Execute` instead.
/// This op performs I/O directly AND returns a TransportRequest (mixed model).
///
/// Inputs:
/// - `path`: String path to write to
/// - `content`: String content to write
///
/// Outputs:
/// - `written_path`: String path that was written
/// - `success`: Bool indicating success
/// - `request`: TransportRequest for transport layer
#[deprecated(
    since = "0.2.0",
    note = "Use PrepareFileWriteOp + TransportOps::Execute instead"
)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WriteFileOp;

impl Executable for WriteFileOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let path = inputs
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'path' string"))?;

        let content = inputs
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'content' string"))?;

        // Create parent directories if needed
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ExecError::new(format!("failed to create directory: {}", e)))?;
        }

        // Write the file
        fs::write(path, content)
            .map_err(|e| ExecError::new(format!("failed to write file '{}': {}", path, e)))?;

        let mut out = HashMap::new();
        out.insert("written_path".to_string(), Value::Str(path.to_string()));
        out.insert("success".to_string(), Value::Bool(true));

        // Also provide a transport request for the transport layer
        let request = TransportRequest::File(FileRequest::write(path, content));
        out.insert("request".to_string(), Value::Request(request));

        Ok(out)
    }
}

/// Execute a shell command (boundary operation).
///
/// **DEPRECATED**: Use `PrepareShellOp` + `TransportOps::Execute` instead.
/// This op performs I/O directly, bypassing the transport layer.
///
/// Inputs:
/// - `command`: String command to execute
/// - `args`: Optional StrList of arguments
/// - `cwd`: Optional working directory
///
/// Outputs:
/// - `stdout`: String stdout output
/// - `stderr`: String stderr output
/// - `exit_code`: Int exit code
/// - `success`: Bool indicating exit_code == 0
#[deprecated(
    since = "0.2.0",
    note = "Use PrepareShellOp + TransportOps::Execute instead"
)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecuteOp;

#[allow(deprecated)]
#[allow(clippy::disallowed_methods)]
impl Executable for ExecuteOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let command = inputs
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'command' string"))?;

        let args = inputs
            .get("args")
            .and_then(|v| v.as_str_list())
            .unwrap_or_default();

        let cwd = inputs.get("cwd").and_then(|v| v.as_str());

        let mut cmd = Command::new(command);
        cmd.args(&args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .map_err(|e| ExecError::new(format!("failed to execute '{}': {}", command, e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1) as i64;

        let mut out = HashMap::new();
        out.insert("stdout".to_string(), Value::Str(stdout));
        out.insert("stderr".to_string(), Value::Str(stderr));
        out.insert("exit_code".to_string(), Value::Int(exit_code));
        out.insert("success".to_string(), Value::Bool(exit_code == 0));

        // Also provide a transport request
        let request = TransportRequest::Shell(ShellRequest {
            command: command.to_string(),
            args,
            cwd: cwd.map(|s| s.to_string()),
            env: std::collections::HashMap::new(),
            stdin: None,
        });
        out.insert("request".to_string(), Value::Request(request));

        Ok(out)
    }
}

/// Make an HTTP request (boundary operation).
///
/// This is a boundary operation - it will be intercepted in dry-run mode.
///
/// Inputs:
/// - `url`: String URL
/// - `method`: String HTTP method (GET, POST, etc.)
/// - `body`: Optional String body
/// - `headers`: Optional MapStrStr of headers
///
/// Outputs:
/// - `status`: Int HTTP status code
/// - `body`: String response body
/// - `success`: Bool indicating 2xx status
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpRequestOp;

impl Executable for HttpRequestOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let url = inputs
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'url' string"))?;

        let method = inputs
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");

        let body = inputs.get("body").and_then(|v| v.as_str());

        let headers = inputs
            .get("headers")
            .and_then(|v| v.as_map_str_str())
            .unwrap_or_default();

        // Parse method
        let http_method = match method.to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            _ => HttpMethod::Get,
        };

        // Build transport request
        let request = TransportRequest::Rest(RestRequest {
            url: url.to_string(),
            method: http_method,
            headers: headers.into_iter().collect(),
            body: body.map(|s| serde_json::Value::String(s.to_string())),
            auth: None,
            query: std::collections::HashMap::new(),
            timeout_ms: None,
        });

        // In a real implementation, we'd execute via transport layer
        // For now, return a placeholder indicating the request was prepared
        let mut out = HashMap::new();
        out.insert("request".to_string(), Value::Request(request));
        out.insert("status".to_string(), Value::Int(0)); // Placeholder
        out.insert("body".to_string(), Value::Str(String::new())); // Placeholder
        out.insert("success".to_string(), Value::Bool(false)); // Not executed yet

        Ok(out)
    }
}

/// List files in a directory, respecting .gitignore.
///
/// Uses `git ls-files` when in a git repository, falls back to recursive
/// directory listing otherwise.
///
/// Inputs:
/// - `repo_path`: Optional String path (defaults to ".")
///
/// Outputs:
/// - `files`: StrList of file paths
/// - `count`: Int number of files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListFilesOp;

impl Executable for ListFilesOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let repo_path = inputs
            .get("repo_path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let files = list_files_impl(repo_path)?;
        let count = files.len() as i64;

        let mut out = HashMap::new();
        out.insert("files".to_string(), Value::StrList(files));
        out.insert("count".to_string(), Value::Int(count));
        Ok(out)
    }
}

/// List files implementation - tries git ls-files, falls back to recursive listing.
///
/// Note: This is a utility function. For DAG nodes that need git,
/// use `node.requires(&cli::GIT)` instead.
#[allow(clippy::disallowed_methods)]
fn list_files_impl(repo_path: &str) -> Result<Vec<String>, ExecError> {
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

/// Recursive directory listing (fallback when not in git repo).
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

/// Read multiple files into a map.
///
/// Returns a map from filename to contents. Skips files that can't be read
/// (binary, permissions, etc.)
///
/// Inputs:
/// - `files`: StrList of file paths
/// - `repo_path`: Optional String base path (defaults to ".")
///
/// Outputs:
/// - `contents`: MapStrStr of filename -> content
/// - `count`: Int number of files read successfully
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadFilesOp;

impl Executable for ReadFilesOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let files = inputs
            .get("files")
            .and_then(|v| v.as_str_list())
            .ok_or_else(|| ExecError::new("missing or invalid 'files' input"))?;

        let repo_path = inputs
            .get("repo_path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let mut contents = std::collections::BTreeMap::new();

        for file in &files {
            let path = Path::new(repo_path).join(file);
            if let Ok(content) = fs::read_to_string(&path) {
                contents.insert(file.clone(), content);
            }
            // Silently skip files that can't be read
        }

        let count = contents.len() as i64;

        let mut out = HashMap::new();
        out.insert("contents".to_string(), Value::MapStrStr(contents));
        out.insert("count".to_string(), Value::Int(count));
        Ok(out)
    }
}

// =============================================================================
// PURE: Prepare Operations (transport pattern - preferred)
// =============================================================================

/// Prepare a file write request (PURE - no I/O).
///
/// This separates the business logic (deciding what to write) from the
/// actual I/O (writing to disk). Use with TransportOps::Execute.
///
/// Inputs:
/// - `path`: String path to write to
/// - `content`: String content to write
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrepareFileWriteOp;

impl Executable for PrepareFileWriteOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        // Accept multiple port names for flexibility, with default
        let path = inputs
            .get("path")
            .or_else(|| inputs.get("output_path"))
            .and_then(|v| v.as_str())
            .unwrap_or("output");  // Default if not provided

        let content = inputs
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'content' string"))?;

        let request = TransportRequest::File(FileRequest::write(path, content));

        let mut out = HashMap::new();
        out.insert("request".to_string(), Value::Request(request));
        Ok(out)
    }
}

/// Prepare a file read request (PURE - no I/O).
///
/// This separates the business logic (deciding what to read) from the
/// actual I/O (reading from disk). Use with TransportOps::Execute.
///
/// Inputs:
/// - `path`: String path to read from
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrepareFileReadOp;

impl Executable for PrepareFileReadOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let path = inputs
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'path' string"))?;

        let request = TransportRequest::File(FileRequest::read(path));

        let mut out = HashMap::new();
        out.insert("request".to_string(), Value::Request(request));
        Ok(out)
    }
}

/// Prepare a file exists check request (PURE - no I/O).
///
/// Inputs:
/// - `path`: String path to check
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrepareFileExistsOp;

impl Executable for PrepareFileExistsOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let path = inputs
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'path' string"))?;

        let request = TransportRequest::File(FileRequest::exists(path));

        let mut out = HashMap::new();
        out.insert("request".to_string(), Value::Request(request));
        Ok(out)
    }
}

/// Prepare a shell command request (PURE - no I/O).
///
/// This separates the business logic (deciding what command to run) from the
/// actual I/O (executing the command). Use with TransportOps::Execute.
///
/// Inputs:
/// - `command`: String command to execute
/// - `args`: Optional StrList of arguments
/// - `cwd`: Optional working directory
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrepareShellOp;

impl Executable for PrepareShellOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let command = inputs
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'command' string"))?;

        let args = inputs
            .get("args")
            .and_then(|v| v.as_str_list())
            .unwrap_or_default();

        let cwd = inputs.get("cwd").and_then(|v| v.as_str());

        let request = TransportRequest::Shell(ShellRequest {
            command: command.to_string(),
            args,
            cwd: cwd.map(|s| s.to_string()),
            env: std::collections::HashMap::new(),
            stdin: None,
        });

        let mut out = HashMap::new();
        out.insert("request".to_string(), Value::Request(request));
        Ok(out)
    }
}

/// Prepare a directory listing request (PURE - no I/O).
///
/// This creates a shell request for `ls` or similar directory listing.
/// Use with TransportOps::Execute.
///
/// Inputs:
/// - `path`: String path to list (defaults to ".")
/// - `recursive`: Optional Bool for recursive listing
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrepareDirectoryListOp;

impl Executable for PrepareDirectoryListOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let path = inputs
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let recursive = inputs
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Use find for recursive, ls for non-recursive
        let (command, args) = if recursive {
            ("find", vec![path.to_string(), "-type".to_string(), "f".to_string()])
        } else {
            ("ls", vec!["-1".to_string(), path.to_string()])
        };

        let request = TransportRequest::Shell(ShellRequest {
            command: command.to_string(),
            args,
            cwd: None,
            env: std::collections::HashMap::new(),
            stdin: None,
        });

        let mut out = HashMap::new();
        out.insert("request".to_string(), Value::Request(request));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_file() {
        // Create a temp file
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "test content").unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let op = ReadFileOp;
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Str(path));

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("exists"), Some(&Value::Bool(true)));
        assert!(result
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap()
            .contains("test content"));
    }

    #[test]
    fn test_read_file_not_exists() {
        let op = ReadFileOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "path".to_string(),
            Value::Str("/nonexistent/file/path".to_string()),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("exists"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_execute_echo() {
        let op = ExecuteOp;
        let mut inputs = HashMap::new();
        inputs.insert("command".to_string(), Value::Str("echo".to_string()));
        inputs.insert(
            "args".to_string(),
            Value::StrList(vec!["hello".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("success"), Some(&Value::Bool(true)));
        assert!(result
            .get("stdout")
            .and_then(|v| v.as_str())
            .unwrap()
            .contains("hello"));
    }
}
