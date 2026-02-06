//! I/O primitives - pure "Prepare" ops that produce `TransportRequest` values.
//!
//! All I/O operations should use the transport pattern:
//! ```text
//! [Prepare*Op] -> [TransportOps::Execute] -> [Parse/Extract]
//!    (pure)          (interceptable)           (pure)
//! ```
//!
//! The Prepare* ops are pure - they build `TransportRequest` values without
//! performing any I/O. The actual I/O happens in `TransportOps::Execute`,
//! which is properly intercepted in DryRun mode.

use gunbc_exec::{
    optional_bool, optional_map_str_str, optional_str, optional_str_list, require_str, ExecError,
    Executable, OutputMap,
};
use gunbc_ir::transport::{
    FileRequest, HttpMethod, RestRequest, ShellRequest, TransportRequest, TransportResponse,
};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prepare an HTTP request (PURE - no I/O).
///
/// Builds a transport request for HTTP/REST operations.
/// Use with TransportOps::Execute.
///
/// Inputs:
/// - `url`: String URL
/// - `method`: String HTTP method (GET, POST, etc.)
/// - `body`: Optional String body
/// - `headers`: Optional Map of headers
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpRequestOp;

impl Executable for HttpRequestOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let url = require_str(&inputs, "url")?;

        let method = optional_str(&inputs, "method").unwrap_or("GET");

        let body = optional_str(&inputs, "body");

        let headers = optional_map_str_str(&inputs, "headers").unwrap_or_default();

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

        OutputMap::new().request("request", request).ok()
    }
}

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
        let path = optional_str(&inputs, "path")
            .or_else(|| optional_str(&inputs, "output_path"))
            .unwrap_or("output"); // Default if not provided

        let content = require_str(&inputs, "content")?;

        let request = TransportRequest::File(FileRequest::write(path, content));

        OutputMap::new().request("request", request).ok()
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
        let path = require_str(&inputs, "path")?;

        let request = TransportRequest::File(FileRequest::read(path));

        OutputMap::new().request("request", request).ok()
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
        let path = require_str(&inputs, "path")?;

        let request = TransportRequest::File(FileRequest::exists(path));

        OutputMap::new().request("request", request).ok()
    }
}

/// Prepare a shell command request (PURE - no I/O).
///
/// This separates the business logic (deciding what command to run) from the
/// actual I/O (executing the command). Use with TransportOps::Execute.
///
/// Inputs:
/// - `command`: String command to execute
/// - `args`: Optional List of arguments
/// - `cwd`: Optional working directory
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrepareShellOp;

impl Executable for PrepareShellOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let command = require_str(&inputs, "command")?;

        let args = optional_str_list(&inputs, "args").unwrap_or_default();

        let cwd = optional_str(&inputs, "cwd");

        let request = TransportRequest::Shell(ShellRequest {
            command: command.to_string(),
            args,
            cwd: cwd.map(|s| s.to_string()),
            env: std::collections::HashMap::new(),
            stdin: None,
        });

        OutputMap::new().request("request", request).ok()
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
        let path = optional_str(&inputs, "path").unwrap_or(".");

        let recursive = optional_bool(&inputs, "recursive").unwrap_or(false);

        // Use find for recursive, ls for non-recursive
        let (command, args) = if recursive {
            (
                "find",
                vec![path.to_string(), "-type".to_string(), "f".to_string()],
            )
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

        OutputMap::new().request("request", request).ok()
    }
}

/// Compare expected content against a file read response (PURE - no I/O).
///
/// This is the "check" phase of a file content upsert. Determines whether
/// a file write can be skipped because disk content already matches.
///
/// Inputs:
/// - `response`: TransportResponse from a file read
/// - `expected_content`: String content that would be written
/// - `check_mode`: Bool (optional) — if true, forces skip=true (verify-only)
///
/// Outputs:
/// - `fresh`: Bool — true if disk content matches expected
/// - `skip`: Bool — true if write should be skipped (fresh || check_mode)
/// - `skip_reason`: String — explanation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompareContentOp;

impl Executable for CompareContentOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let expected = require_str(&inputs, "expected_content")?;
        let check_mode = optional_bool(&inputs, "check_mode").unwrap_or(false);

        // Extract content from the file read response
        let (fresh, detail) = match inputs.get("response") {
            Some(Value::Response(TransportResponse::File(file_resp))) => {
                if !file_resp.success {
                    // Read failed (file doesn't exist or permission error)
                    (false, "file read failed".to_string())
                } else {
                    match &file_resp.content {
                        Some(actual) if actual == expected => {
                            (true, "disk content matches expected".to_string())
                        }
                        Some(_) => (false, "disk content differs from expected".to_string()),
                        None => (false, "file read returned no content".to_string()),
                    }
                }
            }
            Some(Value::Skipped) => (false, "upstream read was skipped".to_string()),
            _ => (false, "missing or invalid file read response".to_string()),
        };

        let skip = fresh || check_mode;
        let skip_reason = if fresh {
            "content is fresh — write skipped".to_string()
        } else if check_mode {
            format!("check mode — would write ({})", detail)
        } else {
            String::new()
        };

        OutputMap::new()
            .bool("fresh", fresh)
            .bool("skip", skip)
            .str("skip_reason", skip_reason)
            .ok()
    }
}

// ============================================================================
// Embedded variants - for hardcoded paths/commands (no input ports needed)
// ============================================================================

/// Prepare a file exists check with embedded path (PURE - no I/O).
///
/// Use this when the path is known at graph construction time.
/// For dynamic paths from upstream nodes, use `PrepareFileExistsOp` instead.
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedFileExistsOp {
    pub path: String,
}

impl EmbeddedFileExistsOp {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

impl Executable for EmbeddedFileExistsOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let request = TransportRequest::File(FileRequest::exists(&self.path));

        OutputMap::new().request("request", request).ok()
    }
}

/// Prepare a shell command with embedded command/args (PURE - no I/O).
///
/// Use this when the command is known at graph construction time.
/// For dynamic commands from upstream nodes, use `PrepareShellOp` instead.
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedShellOp {
    pub command: String,
    pub args: Vec<String>,
}

impl EmbeddedShellOp {
    pub fn new(command: &str, args: &[&str]) -> Self {
        Self {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Create from a config-style command slice (first element is command, rest are args).
    pub fn from_config(config_cmd: &[&str]) -> Self {
        if config_cmd.is_empty() {
            Self {
                command: String::new(),
                args: Vec::new(),
            }
        } else {
            Self {
                command: config_cmd[0].to_string(),
                args: config_cmd[1..].iter().map(|s| s.to_string()).collect(),
            }
        }
    }
}

impl Executable for EmbeddedShellOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let request = TransportRequest::Shell(ShellRequest {
            command: self.command.clone(),
            args: self.args.clone(),
            cwd: None,
            env: std::collections::HashMap::new(),
            stdin: None,
        });

        OutputMap::new().request("request", request).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{FileOp, FileResponse};

    // ========================================================================
    // CompareContentOp tests
    // ========================================================================

    #[test]
    fn test_compare_content_match() {
        let op = CompareContentOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("hello world".into()),
                exists: None,
                error: None,
            })),
        );
        inputs.insert(
            "expected_content".to_string(),
            Value::Str("hello world".into()),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("fresh"), Some(&Value::Bool(true)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_compare_content_mismatch() {
        let op = CompareContentOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("old content".into()),
                exists: None,
                error: None,
            })),
        );
        inputs.insert(
            "expected_content".to_string(),
            Value::Str("new content".into()),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("fresh"), Some(&Value::Bool(false)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_compare_content_file_missing() {
        let op = CompareContentOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: false,
                content: None,
                exists: None,
                error: Some("No such file".into()),
            })),
        );
        inputs.insert(
            "expected_content".to_string(),
            Value::Str("content".into()),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("fresh"), Some(&Value::Bool(false)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_compare_content_check_mode_forces_skip() {
        let op = CompareContentOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("old content".into()),
                exists: None,
                error: None,
            })),
        );
        inputs.insert(
            "expected_content".to_string(),
            Value::Str("new content".into()),
        );
        inputs.insert("check_mode".to_string(), Value::Bool(true));

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("fresh"), Some(&Value::Bool(false)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(true)));
        // skip_reason should mention check mode
        let reason = result.get("skip_reason").and_then(|v| v.as_str()).unwrap();
        assert!(reason.contains("check mode"));
    }

    #[test]
    fn test_compare_content_check_mode_fresh() {
        let op = CompareContentOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("same".into()),
                exists: None,
                error: None,
            })),
        );
        inputs.insert("expected_content".to_string(), Value::Str("same".into()));
        inputs.insert("check_mode".to_string(), Value::Bool(true));

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("fresh"), Some(&Value::Bool(true)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(true)));
    }

    // ========================================================================
    // Existing tests
    // ========================================================================

    #[test]
    fn test_prepare_file_write() {
        let op = PrepareFileWriteOp;
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Str("test.txt".to_string()));
        inputs.insert("content".to_string(), Value::Str("hello".to_string()));

        let result = op.execute(inputs).unwrap();
        assert!(result.contains_key("request"));
    }

    #[test]
    fn test_prepare_file_read() {
        let op = PrepareFileReadOp;
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Str("test.txt".to_string()));

        let result = op.execute(inputs).unwrap();
        assert!(result.contains_key("request"));
    }

    #[test]
    fn test_prepare_file_exists() {
        let op = PrepareFileExistsOp;
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Str("test.txt".to_string()));

        let result = op.execute(inputs).unwrap();
        assert!(result.contains_key("request"));
    }

    #[test]
    fn test_prepare_shell() {
        let op = PrepareShellOp;
        let mut inputs = HashMap::new();
        inputs.insert("command".to_string(), Value::Str("echo".to_string()));
        inputs.insert(
            "args".to_string(),
            Value::str_list(vec!["hello".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        assert!(result.contains_key("request"));
    }

    #[test]
    fn test_prepare_directory_list() {
        let op = PrepareDirectoryListOp;
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Str(".".to_string()));

        let result = op.execute(inputs).unwrap();
        assert!(result.contains_key("request"));
    }

    #[test]
    fn test_http_request() {
        let op = HttpRequestOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "url".to_string(),
            Value::Str("https://example.com".to_string()),
        );
        inputs.insert("method".to_string(), Value::Str("GET".to_string()));

        let result = op.execute(inputs).unwrap();
        assert!(result.contains_key("request"));
    }
}
