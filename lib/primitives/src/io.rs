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
    optional_bool_strict, optional_map_str_str_strict, optional_str_list_strict,
    optional_str_strict, require_str, ExecError, Executable, OutputMap,
};
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
/// - `headers`: Optional Map of headers
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpRequestOp;

impl Executable for HttpRequestOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let url = require_str(&inputs, "url")?;

        let http_method = match optional_str_strict(&inputs, "method")? {
            None => HttpMethod::Get,
            Some(method) => HttpMethod::parse(method)
                .ok_or_else(|| ExecError::new(format!("unsupported http method '{}'", method)))?,
        };

        let body = optional_str_strict(&inputs, "body")?;

        let headers = optional_map_str_str_strict(&inputs, "headers")?.unwrap_or_default();

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

        OutputMap::new()
            .request("request", request)
            .bool("skip", false)
            .ok()
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
        let path = require_str(&inputs, "path")?;

        let content = require_str(&inputs, "content")?;

        let request = TransportRequest::File(FileRequest::write(path, content));

        OutputMap::new()
            .request("request", request)
            .bool("skip", false)
            .ok()
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

        OutputMap::new()
            .request("request", request)
            .bool("skip", false)
            .ok()
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

        OutputMap::new()
            .request("request", request)
            .bool("skip", false)
            .ok()
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

        let args = optional_str_list_strict(&inputs, "args")?.unwrap_or_default();

        let cwd = optional_str_strict(&inputs, "cwd")?;

        let mut req = ShellRequest::new(command).args(args);
        if let Some(dir) = cwd {
            req = req.cwd(dir);
        }
        let request = req.into_transport_request();

        OutputMap::new()
            .request("request", request)
            .bool("skip", false)
            .ok()
    }
}

/// Prepare a directory listing request (PURE - no I/O).
///
/// This creates a shell request for `ls` or similar directory listing.
/// Use with TransportOps::Execute.
///
/// Inputs:
/// - `path`: String path to list
/// - `recursive`: Optional Bool for recursive listing
///
/// Outputs:
/// - `request`: TransportRequest for transport layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrepareDirectoryListOp;

impl Executable for PrepareDirectoryListOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let path = require_str(&inputs, "path")?;

        let recursive = optional_bool_strict(&inputs, "recursive")?.unwrap_or(false);

        // Use find for recursive, ls for non-recursive
        let (command, args) = if recursive {
            (
                "find",
                vec![path.to_string(), "-type".to_string(), "f".to_string()],
            )
        } else {
            ("ls", vec!["-1".to_string(), path.to_string()])
        };

        let request = ShellRequest::new(command)
            .args(args)
            .into_transport_request();

        OutputMap::new()
            .request("request", request)
            .bool("skip", false)
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

        OutputMap::new()
            .request("request", request)
            .bool("skip", false)
            .ok()
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
}

impl Executable for EmbeddedShellOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let request = ShellRequest::new(&self.command)
            .args(self.args.iter().map(|s| s.as_str()))
            .into_transport_request();

        OutputMap::new()
            .request("request", request)
            .bool("skip", false)
            .ok()
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
    fn test_prepare_file_write_requires_path() {
        let op = PrepareFileWriteOp;
        let mut inputs = HashMap::new();
        inputs.insert("content".to_string(), Value::Str("hello".to_string()));

        let err = op.execute(inputs).unwrap_err();
        assert!(err.to_string().contains("path"));
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

    #[test]
    fn test_http_request_rejects_invalid_method() {
        let op = HttpRequestOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "url".to_string(),
            Value::Str("https://example.com".to_string()),
        );
        inputs.insert("method".to_string(), Value::Str("POTS".to_string()));

        let err = op.execute(inputs).unwrap_err();
        assert!(err.to_string().contains("unsupported http method"));
    }
}
