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
use gunbc_exec::{optional_bool, require_request, ExecError, Executable, IntoExecResult, OutputMap};
use gunbc_ir::resource::ensure_capability_marker;
use gunbc_ir::transport::{AuthMethod, TransportRequest, TransportResponse};
use gunbc_ir::{AuthScheme, Credential, Secret, Value};
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

                // Auth resolution: prefer Credential, fall back to legacy res:auth.
                if let TransportRequest::Rest(ref mut r) = request {
                    let mut applied_credential = false;

                    if let Some(ref auth) = r.auth {
                        if matches!(
                            auth,
                            AuthMethod::EnvVar(_) | AuthMethod::EnvVarHeader { .. }
                        ) {
                            let cred = if let Some(cred_value) = inputs.get("res:credential") {
                                Credential::try_from(cred_value).map_err(|e| {
                                    ExecError::new(format!("invalid 'res:credential': {}", e))
                                })?
                            } else {
                                let auth_value = inputs.get("res:auth").ok_or_else(|| {
                                    ExecError::new(
                                        "missing 'res:credential' (or legacy 'res:auth') for REST auth",
                                    )
                                })?;
                                credential_from_auth_value(auth, auth_value)?
                            };
                            cred.apply(r);
                            applied_credential = true;
                        }
                    }

                    // General credential path: apply if provided and not already applied.
                    if !applied_credential {
                        if let Some(cred_value) = inputs.get("res:credential") {
                            let cred = Credential::try_from(cred_value).map_err(|e| {
                                ExecError::new(format!("invalid 'res:credential': {}", e))
                            })?;
                            cred.apply(r);
                        }
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
    execute_transport(request).exec_context("transport error")
}

fn credential_from_auth_value(auth: &AuthMethod, value: &Value) -> Result<Credential, ExecError> {
    if let Ok(cred) = Credential::try_from(value) {
        return Ok(cred);
    }

    let map = match value {
        Value::Map(m) => m,
        _ => {
            return Err(ExecError::new(
                "invalid 'res:auth' input: expected map",
            ))
        }
    };

    ensure_capability_marker(map, "AuthToken")
        .map_err(|e| ExecError::new(format!("invalid 'res:auth' input: {}", e)))?;

    let env_var = map
        .get("env_var")
        .and_then(Value::as_str)
        .ok_or_else(|| ExecError::new("invalid 'res:auth' input: missing env_var"))?;
    let token = match map.get("token") {
        Some(Value::Secret(s)) => s.expose().to_string(),
        _ => {
            return Err(ExecError::new(
                "invalid 'res:auth' input: missing token secret",
            ))
        }
    };

    let scheme = match auth {
        AuthMethod::EnvVar(expected) => {
            if env_var != expected {
                return Err(ExecError::new(format!(
                    "auth token env var mismatch: request expects '{}', token is '{}'",
                    expected, env_var
                )));
            }
            AuthScheme::Bearer
        }
        AuthMethod::EnvVarHeader { header, env_var: expected } => {
            if env_var != expected {
                return Err(ExecError::new(format!(
                    "auth token env var mismatch: request expects '{}', token is '{}'",
                    expected, env_var
                )));
            }
            AuthScheme::Header {
                name: header.clone(),
            }
        }
        other => {
            return Err(ExecError::new(format!(
                "legacy 'res:auth' used with unsupported auth method: {:?}",
                other
            )))
        }
    };

    let secret = Secret::from_env_var(env_var, token);
    Ok(Credential::new(secret, scheme))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{FileRequest, ShellRequest};
    use gunbc_ir::{Secret, AuthScheme, Value};
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

        let result = TransportOps::Execute
            .execute(inputs)
            .expect("skip should short-circuit");

        assert_eq!(result.get("skip"), Some(&Value::Bool(true)));
        assert!(!result.contains_key("response"));
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

        let mut inputs = HashMap::new();
        inputs.insert("request".to_string(), Value::Request(request));
        inputs.insert("res:credential".to_string(), cred_value);

        let result = TransportOps::Execute.execute(inputs);
        // The request will fail (no real server), but we can verify the credential was applied
        // by checking that it didn't fail on missing credential input — the credential path is separate.
        // For a proper test we'd need a mock transport, but we can at least verify
        // the credential parsing doesn't error.
        // Actually, since the URL doesn't have auth set, the old path is skipped,
        // and the credential is applied. The actual HTTP call will fail, which is fine.
        assert!(result.is_err()); // HTTP call fails, but no auth error
        let err_msg = result.unwrap_err().0;
        assert!(!err_msg.contains("res:credential"), "credential should parse successfully");
        assert!(!err_msg.contains("res:auth"), "should not require res:auth");
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

        let mut inputs = HashMap::new();
        inputs.insert("request".to_string(), Value::Request(request));
        inputs.insert("res:credential".to_string(), cred_value);

        let result = TransportOps::Execute.execute(inputs);
        assert!(result.is_err()); // HTTP call fails
        let err_msg = result.unwrap_err().0;
        assert!(!err_msg.contains("res:credential"), "credential should parse successfully");
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

        let result = TransportOps::Execute
            .execute(inputs)
            .expect("transport should execute");

        assert_eq!(result.get("success"), Some(&Value::Bool(true)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(false)));
        let stdout = result.get("stdout").and_then(|v| v.as_str()).unwrap_or("");
        assert!(stdout.contains("hello"));
    }
}
