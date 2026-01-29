//! Gist operations.
//!
//! Operations for working with GitHub Gists.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_lib_gist_ops::prepare_gist_request;
//!
//! let request = prepare_gist_request("# My Content", true, "My gist");
//! // request is now a TransportRequest ready to be executed
//! ```

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::gist::GistRequest;
use gunbc_ir::transport::{ShellResponse, TransportRequest, TransportResponse};
use gunbc_ir::Value;
use gunbc_lib_transport::execute_transport;
use std::collections::HashMap;

/// Gist operations for use in DAG nodes.
#[derive(Debug, Clone)]
pub enum GistOps {
    /// Prepare a gist creation request (PURE - no I/O)
    PrepareRequest { public: bool },
    /// Execute a transport request (BOUNDARY)
    ExecuteTransport,
}

impl Executable for GistOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GistOps::PrepareRequest { public } => {
                let markdown = inputs
                    .get("markdown")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExecError::new("missing or invalid 'markdown' input"))?;

                let request =
                    prepare_gist_request(markdown, *public, "Code snapshot created by gunbc-gist");

                let mut out = HashMap::new();
                out.insert("request".to_string(), Value::Request(request));
                Ok(out)
            }
            GistOps::ExecuteTransport => {
                let request = inputs
                    .get("request")
                    .and_then(|v| v.as_request())
                    .ok_or_else(|| ExecError::new("missing or invalid 'request' input"))?;

                let response = execute_transport(&request)
                    .map_err(|e| ExecError::new(format!("transport error: {}", e)))?;

                let url = extract_gist_url(&response);

                let mut out = HashMap::new();
                out.insert("response".to_string(), Value::Response(response));
                out.insert("url".to_string(), Value::Str(url));
                Ok(out)
            }
        }
    }
}

// ============================================================================
// Standalone helper functions
// ============================================================================

/// Prepare a gist creation request.
///
/// Returns a `TransportRequest` that can be executed to create a gist.
/// This is PURE - it doesn't perform any I/O, just builds the request.
///
/// # Example
///
/// ```ignore
/// let request = prepare_gist_request("# Hello", true, "My public gist");
/// // Execute with gunbc_transport::execute_transport(&request)
/// ```
pub fn prepare_gist_request(
    content: &str,
    public: bool,
    description: &str,
) -> TransportRequest {
    GistRequest::new()
        .file("snapshot.md", content)
        .public(public)
        .description(description)
        .to_shell_request()
}

/// Extract gist URL from a transport response.
pub fn extract_gist_url(response: &TransportResponse) -> String {
    match response {
        TransportResponse::Shell(ShellResponse { stdout, .. }) => {
            gunbc_ir::transport::gist::parse_gist_url_from_shell(stdout)
                .unwrap_or_else(|| stdout.trim().to_string())
        }
        TransportResponse::Rest(r) => {
            gunbc_ir::transport::gist::parse_gist_url_from_rest(&r.body)
                .unwrap_or_else(|| "unknown".to_string())
        }
        _ => "unknown".to_string(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_gist_request() {
        let request = prepare_gist_request("# Test", false, "Test gist");

        match request {
            TransportRequest::Shell(req) => {
                assert_eq!(req.command, "gh");
                assert!(req.args.contains(&"gist".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        assert!(result.contains_key("request"));
        match result.get("request") {
            Some(Value::Request(TransportRequest::Shell(_))) => {}
            _ => panic!("expected shell request"),
        }
    }
}
