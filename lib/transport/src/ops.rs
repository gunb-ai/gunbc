//! Transport operations.
//!
//! Unified transport execution for any I/O operation.
//! This is the standard way to execute transport requests - use this
//! instead of implementing your own transport wrapper.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_lib_transport::{TransportOps, execute_request};
//!
//! // In a DAG node
//! let node = Node::opaque("execute", inputs, outputs, TransportOps::Execute);
//!
//! // Or call directly
//! let response = execute_request(&request)?;
//! ```

use crate::executor::execute_transport;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Transport operations for use in DAG nodes.
#[derive(Debug, Clone)]
pub enum TransportOps {
    /// Execute any transport request (BOUNDARY - world I/O)
    Execute,
}

impl Executable for TransportOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            TransportOps::Execute => {
                let request = inputs
                    .get("request")
                    .and_then(|v| v.as_request())
                    .ok_or_else(|| ExecError::new("missing or invalid 'request' input"))?;

                let response = execute_request(&request)?;

                let mut out = HashMap::new();
                
                // Extract extra info for file responses
                if let TransportResponse::File(file_resp) = &response {
                    out.insert("written_path".to_string(), Value::Str(file_resp.path.clone()));
                    if let Some(content) = &file_resp.content {
                        out.insert("content".to_string(), Value::Str(content.clone()));
                    }
                }

                // Extract extra info for shell responses
                if let TransportResponse::Shell(shell_resp) = &response {
                    out.insert("stdout".to_string(), Value::Str(shell_resp.stdout.clone()));
                    out.insert("stderr".to_string(), Value::Str(shell_resp.stderr.clone()));
                    out.insert("exit_code".to_string(), Value::Int(shell_resp.exit_code as i64));
                    out.insert("success".to_string(), Value::Bool(shell_resp.success()));
                }
                
                out.insert("response".to_string(), Value::Response(response));
                Ok(out)
            }
        }
    }
}

// ============================================================================
// Standalone helper functions
// ============================================================================

/// Execute a transport request.
///
/// This is the standard way to execute any I/O operation in gunbc.
/// Returns the transport response.
///
/// # Example
///
/// ```ignore
/// let request = TransportRequest::File(FileRequest::read("config.toml"));
/// let response = execute_request(&request)?;
/// ```
pub fn execute_request(request: &TransportRequest) -> Result<TransportResponse, ExecError> {
    execute_transport(request).map_err(|e| ExecError::new(format!("transport error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{FileRequest, ShellRequest};
    use gunbc_ir::Value;
    use std::collections::HashMap;

    #[test]
    fn test_transport_ops_requires_request() {
        let op = TransportOps::Execute;
        let result = op.execute(HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_request_file_exists() {
        // Test with a file that exists
        let request = TransportRequest::File(FileRequest::exists("Cargo.toml"));
        let response = execute_request(&request);
        assert!(response.is_ok());
    }

    #[test]
    fn test_transport_ops_shell_outputs() {
        let request = TransportRequest::Shell(ShellRequest {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            cwd: None,
            env: HashMap::new(),
            stdin: None,
        });

        let mut inputs = HashMap::new();
        inputs.insert("request".to_string(), Value::Request(request));

        let result = TransportOps::Execute.execute(inputs).expect("transport should execute");

        assert_eq!(result.get("success"), Some(&Value::Bool(true)));
        let stdout = result
            .get("stdout")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(stdout.contains("hello"));
    }
}
