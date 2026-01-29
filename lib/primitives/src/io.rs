//! I/O primitives - world interactions (boundaries).
//!
//! These operations interact with the outside world and are automatically
//! identified as boundaries for dry-run interception.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{FileRequest, HttpMethod, RestRequest, ShellRequest, TransportRequest};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Read a file from the filesystem.
///
/// Inputs:
/// - `path`: String path to the file
///
/// Outputs:
/// - `content`: String contents of the file
/// - `exists`: Bool indicating if the file exists
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
/// This is a boundary operation - it will be intercepted in dry-run mode.
///
/// Inputs:
/// - `path`: String path to write to
/// - `content`: String content to write
///
/// Outputs:
/// - `written_path`: String path that was written
/// - `success`: Bool indicating success
/// - `request`: TransportRequest for transport layer
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
/// This is a boundary operation - it will be intercepted in dry-run mode.
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecuteOp;

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
