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

use crate::backend::execute_transport_with_backend;
use gunbc_exec::{require_bool, require_request, ExecError, Executable, IntoExecResult, OutputMap};
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use gunbc_ir::{Credential, Value};
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
                let skip = require_bool(&inputs, "skip")?;

                if skip {
                    let mut out = OutputMap::new()
                        .bool("skip", true)
                        .value("response", Value::Skipped);
                    if let Some(reason) = inputs.get("skip_reason") {
                        out = out.value("skip_reason", reason.clone());
                    }
                    return out.ok();
                }

                let mut request = require_request(&inputs, "request")?;

                // Apply credentials if provided.
                if let TransportRequest::Rest(ref mut r) = request {
                    if let Some(cred_value) = inputs.get("res:credential") {
                        let cred = Credential::try_from(cred_value).map_err(|e| {
                            ExecError::new(format!("invalid 'res:credential': {}", e))
                        })?;
                        cred.apply(r);
                    } else if let Some(cred) = r.auth.take() {
                        cred.apply(r);
                    }
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

                out.response("response", response).bool("skip", false).ok()
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
    execute_transport_with_backend(request).exec_context("transport error")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TransportError;
    use crate::{TransportBackend, TransportBackendGuard};
    use gunbc_ir::transport::{FileRequest, RestResponse, ShellRequest, TransportRequest, TransportResponse};
    use gunbc_ir::{AuthScheme, Secret, Value};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone)]
    struct CaptureBackend {
        captured: Arc<Mutex<Option<TransportRequest>>>,
    }

    impl TransportBackend for CaptureBackend {
        fn execute(&self, request: &TransportRequest) -> Result<TransportResponse, TransportError> {
            *self.captured.lock().expect("capture lock") = Some(request.clone());
            match request {
                TransportRequest::Rest(_) => Ok(TransportResponse::Rest(RestResponse::ok(
                    serde_json::json!({}),
                ))),
                _ => Err(TransportError::new("unexpected transport request")),
            }
        }
    }

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

        let result = TransportOps::Execute
            .execute(inputs)
            .expect("skip should short-circuit");

        assert_eq!(result.get("skip"), Some(&Value::Bool(true)));
        assert_eq!(result.get("response"), Some(&Value::Skipped));
    }

    #[test]
    fn test_res_credential_applies_to_rest_request() {
        use gunbc_ir::transport::rest::RestRequest;

        // Build a REST request wrapped in a TransportRequest
        let rest_req = RestRequest::get("https://api.example.com/test");
        let request = TransportRequest::Rest(rest_req);

        // Build a Credential (bearer)
        let cred = Credential::new(Secret::static_value("sk-test-123"), AuthScheme::Bearer);
        let cred_value: Value = cred.into();

        let captured = Arc::new(Mutex::new(None));
        let backend = Arc::new(CaptureBackend {
            captured: captured.clone(),
        });
        let _guard = TransportBackendGuard::install(backend);

        let mut inputs = HashMap::new();
        inputs.insert("request".to_string(), Value::Request(request));
        inputs.insert("res:credential".to_string(), cred_value);
        inputs.insert("skip".to_string(), Value::Bool(false));

        let result = TransportOps::Execute
            .execute(inputs)
            .expect("transport should execute");
        assert_eq!(result.get("skip"), Some(&Value::Bool(false)));

        let captured_req = captured
            .lock()
            .expect("capture lock")
            .clone()
            .expect("captured request");
        let rest = match captured_req {
            TransportRequest::Rest(r) => r,
            other => panic!("expected REST request, got {:?}", other),
        };
        assert_eq!(
            rest.headers.get("Authorization"),
            Some(&"Bearer sk-test-123".to_string())
        );
        assert!(rest.auth.is_none(), "auth should be cleared after apply()");
    }

    #[test]
    fn test_res_credential_header_scheme() {
        use gunbc_ir::transport::rest::RestRequest;

        let rest_req = RestRequest::get("https://api.example.com/test");
        let request = TransportRequest::Rest(rest_req);

        let cred = Credential::new(
            Secret::static_value("sk-ant-key"),
            AuthScheme::Header { name: "x-api-key".to_string() },
        );
        let cred_value: Value = cred.into();

        let captured = Arc::new(Mutex::new(None));
        let backend = Arc::new(CaptureBackend {
            captured: captured.clone(),
        });
        let _guard = TransportBackendGuard::install(backend);

        let mut inputs = HashMap::new();
        inputs.insert("request".to_string(), Value::Request(request));
        inputs.insert("res:credential".to_string(), cred_value);
        inputs.insert("skip".to_string(), Value::Bool(false));

        let result = TransportOps::Execute
            .execute(inputs)
            .expect("transport should execute");
        assert_eq!(result.get("skip"), Some(&Value::Bool(false)));

        let captured_req = captured
            .lock()
            .expect("capture lock")
            .clone()
            .expect("captured request");
        let rest = match captured_req {
            TransportRequest::Rest(r) => r,
            other => panic!("expected REST request, got {:?}", other),
        };
        assert_eq!(
            rest.headers.get("x-api-key"),
            Some(&"sk-ant-key".to_string())
        );
        assert!(rest.auth.is_none(), "auth should be cleared after apply()");
    }

    #[test]
    fn test_transport_ops_shell_outputs() {
        let request = ShellRequest::new("echo")
            .arg("hello")
            .into_transport_request();

        let mut inputs = HashMap::new();
        inputs.insert("request".to_string(), Value::Request(request));
        inputs.insert("skip".to_string(), Value::Bool(false));

        let result = TransportOps::Execute
            .execute(inputs)
            .expect("transport should execute");

        assert_eq!(result.get("success"), Some(&Value::Bool(true)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(false)));
        let stdout = result.get("stdout").and_then(|v| v.as_str()).unwrap_or("");
        assert!(stdout.contains("hello"));
    }
}
