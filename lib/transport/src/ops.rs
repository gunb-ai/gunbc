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
use gunbc_exec::{
    optional_bool_strict, optional_int_strict, optional_str_strict, require_int, require_request,
    require_response, require_str, ExecError, Executable, IntoExecResult, OutputMap,
    TransportResponseExt,
};
use gunbc_ir::transport::{TcpRequest, TransportRequest, TransportResponse};
use gunbc_ir::{Credential, Value};
use std::collections::HashMap;

/// Transport operations for use in DAG nodes.
#[derive(Debug, Clone)]
pub enum TransportOps {
    /// Execute any transport request (BOUNDARY - world I/O)
    Execute,
    /// Prepare typed TCP request fields into a transport request.
    PrepareTcp,
    /// Parse a TCP transport response into typed fields.
    ParseTcpResponse,
}

impl Executable for TransportOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            TransportOps::PrepareTcp => execute_prepare_tcp(inputs),
            TransportOps::ParseTcpResponse => execute_parse_tcp_response(inputs),
            TransportOps::Execute => {
                let skip = optional_bool_strict(&inputs, "skip")?.unwrap_or(false);

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

fn execute_prepare_tcp(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let host = require_str(&inputs, "host")?.to_string();
    let port_value = require_int(&inputs, "port")?;
    let port = u16::try_from(port_value).map_err(|_| {
        ExecError::new(format!(
            "invalid 'port' input: expected 0..65535, got {port_value}"
        ))
    })?;
    let mut request = TcpRequest::new(host, port);

    if let Some(data) = optional_str_strict(&inputs, "data")? {
        request = request.data(data);
    }
    if let Some(timeout) = optional_int_strict(&inputs, "read_timeout_ms")? {
        let timeout = u64::try_from(timeout).map_err(|_| {
            ExecError::new(format!(
                "invalid 'read_timeout_ms' input: expected >= 0, got {timeout}"
            ))
        })?;
        request = request.read_timeout(timeout);
    }
    if let Some(timeout) = optional_int_strict(&inputs, "write_timeout_ms")? {
        let timeout = u64::try_from(timeout).map_err(|_| {
            ExecError::new(format!(
                "invalid 'write_timeout_ms' input: expected >= 0, got {timeout}"
            ))
        })?;
        request = request.write_timeout(timeout);
    }

    OutputMap::new()
        .request("request", TransportRequest::Tcp(request))
        .bool("skip", false)
        .ok()
}

fn execute_parse_tcp_response(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = gunbc_exec::propagate_skipped(
        &inputs,
        "response",
        &[
            "connected",
            "bytes_sent",
            "bytes_received",
            "data",
            "error",
            "success",
            "error_summary",
            "detail",
        ],
    ) {
        return result;
    }

    let response = require_response(&inputs, "response")?;
    let tcp = response.require_tcp()?;
    let detail = if let Some(error) = tcp.error.as_deref() {
        error.to_string()
    } else {
        format!(
            "connected={} bytes_sent={} bytes_received={}",
            tcp.connected, tcp.bytes_sent, tcp.bytes_received
        )
    };

    let mut out = OutputMap::new()
        .bool("connected", tcp.connected)
        .int("bytes_sent", tcp.bytes_sent as i64)
        .int("bytes_received", tcp.bytes_received as i64)
        .status(tcp.is_ok(), tcp.error.clone().unwrap_or_default(), detail);
    if let Some(data) = &tcp.data {
        out = out.str("data", data.clone());
    }
    if let Some(error) = &tcp.error {
        out = out.str("error", error.clone());
    }
    out.ok()
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
    use gunbc_ir::transport::{
        FileRequest, RestResponse, ShellRequest, TcpResponse, TransportRequest, TransportResponse,
    };
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
            AuthScheme::Header {
                name: "x-api-key".to_string(),
            },
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

    #[test]
    fn test_prepare_tcp_builds_typed_transport_request() {
        let mut inputs = HashMap::new();
        inputs.insert("host".to_string(), Value::Str("127.0.0.1".to_string()));
        inputs.insert("port".to_string(), Value::Int(9000));
        inputs.insert("data".to_string(), Value::Str("PING\n".to_string()));
        inputs.insert("read_timeout_ms".to_string(), Value::Int(250));
        inputs.insert("write_timeout_ms".to_string(), Value::Int(400));

        let result = TransportOps::PrepareTcp
            .execute(inputs)
            .expect("prepare tcp should succeed");
        assert_eq!(result.get("skip"), Some(&Value::Bool(false)));
        let request = result
            .get("request")
            .and_then(Value::as_request)
            .expect("request output");
        match request {
            TransportRequest::Tcp(tcp) => {
                assert_eq!(tcp.host, "127.0.0.1");
                assert_eq!(tcp.port, 9000);
                assert_eq!(tcp.data.as_deref(), Some("PING\n"));
                assert_eq!(tcp.read_timeout_ms, Some(250));
                assert_eq!(tcp.write_timeout_ms, Some(400));
            }
            other => panic!("expected tcp request, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_tcp_response_extracts_typed_outputs() {
        let mut inputs = HashMap::new();
        inputs.insert("skip".to_string(), Value::Bool(false));
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Tcp(TcpResponse::ok(
                Some("PONG\n".to_string()),
                5,
                5,
            ))),
        );

        let result = TransportOps::ParseTcpResponse
            .execute(inputs)
            .expect("parse tcp should succeed");
        assert_eq!(result.get("connected"), Some(&Value::Bool(true)));
        assert_eq!(result.get("bytes_sent"), Some(&Value::Int(5)));
        assert_eq!(result.get("bytes_received"), Some(&Value::Int(5)));
        assert_eq!(result.get("success"), Some(&Value::Bool(true)));
        assert_eq!(
            result.get("error_summary").and_then(Value::as_str),
            Some("")
        );
        assert_eq!(result.get("data").and_then(Value::as_str), Some("PONG\n"));
    }
}
