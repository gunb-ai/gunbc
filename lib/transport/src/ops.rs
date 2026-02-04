//! Transport operations.
//!
//! Unified transport execution for any I/O operation via `TransportOps::Execute` nodes.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_lib_transport::TransportOps;
//!
//! // In a DAG node - this is the ONLY way to do I/O
//! let node = Node::opaque("execute", inputs, outputs, TransportOps::Execute);
//! ```
//!
//! Note: `execute_request()` is internal to this crate and not exported.
//! This structural enforcement ensures all I/O goes through visible DAG nodes.

use crate::executor::execute_transport;
use gunbc_exec::{optional_bool, require_request, ExecError, Executable, OutputMap};
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
                let skip = optional_bool(&inputs, "skip").unwrap_or(false);

                if skip {
                    let mut out = OutputMap::new().bool("skip", true);
                    if let Some(reason) = inputs.get("skip_reason") {
                        out = out.value("skip_reason", reason.clone());
                    }
                    return out.ok();
                }

                let mut request = require_request(&inputs, "request")?;

                // DI violation: env vars resolved inline via std::env::var.
                // Phase 2 will pass resolved auth through DAG input ports.
                if let TransportRequest::Rest(ref mut r) = request {
                    r.resolve_auth(|var| std::env::var(var).ok());
                }

                let response = execute_request(&request)?;

                let mut out = OutputMap::new();

                // Extract extra info for file responses
                if let TransportResponse::File(file_resp) = &response {
                    out = out.str("written_path", file_resp.path.clone());
                    if let Some(content) = &file_resp.content {
                        out = out.str("content", content.clone());
                    }
                }

                // Extract extra info for shell responses
                if let TransportResponse::Shell(shell_resp) = &response {
                    out = out
                        .str("stdout", shell_resp.stdout.clone())
                        .str("stderr", shell_resp.stderr.clone())
                        .int("exit_code", shell_resp.exit_code as i64)
                        .bool("success", shell_resp.success());
                }

                out.response("response", response)
                    .bool("skip", false)
                    .ok()
            }
        }
    }
}

// ============================================================================
// Standalone helper functions
// ============================================================================

/// Execute a transport request (internal).
///
/// This function is NOT exported - it's only callable from within this crate.
/// External code must use `TransportOps::Execute` nodes in a DAG.
pub(crate) fn execute_request(request: &TransportRequest) -> Result<TransportResponse, ExecError> {
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
    fn test_transport_ops_skip_without_request() {
        let mut inputs = HashMap::new();
        inputs.insert("skip".to_string(), Value::Bool(true));

        let result = TransportOps::Execute.execute(inputs).expect("skip should short-circuit");

        assert_eq!(result.get("skip"), Some(&Value::Bool(true)));
        assert!(!result.contains_key("response"));
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
        assert_eq!(result.get("skip"), Some(&Value::Bool(false)));
        let stdout = result
            .get("stdout")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(stdout.contains("hello"));
    }
}
