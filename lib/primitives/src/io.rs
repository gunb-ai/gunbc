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

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{FileRequest, HttpMethod, RestRequest, ShellRequest, TransportRequest};
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
/// - `headers`: Optional MapStrStr of headers
///
/// Outputs:
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

        let mut out = HashMap::new();
        out.insert("request".to_string(), Value::Request(request));
        Ok(out)
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
        let path = inputs
            .get("path")
            .or_else(|| inputs.get("output_path"))
            .and_then(|v| v.as_str())
            .unwrap_or("output"); // Default if not provided

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

        let mut out = HashMap::new();
        out.insert("request".to_string(), Value::Request(request));
        Ok(out)
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
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let request = TransportRequest::File(FileRequest::exists(&self.path));

        let mut out = HashMap::new();
        out.insert("request".to_string(), Value::Request(request));
        Ok(out)
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
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let request = TransportRequest::Shell(ShellRequest {
            command: self.command.clone(),
            args: self.args.clone(),
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
            Value::StrList(vec!["hello".to_string()]),
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
