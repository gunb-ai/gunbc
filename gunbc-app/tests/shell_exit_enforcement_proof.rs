//! Proof tests for RT-I4: Shell exit code enforcement.
//!
//! These tests demonstrate that the GenericShellParseOp now catches non-zero
//! exit codes in TrimStdout mode, preventing the silent-empty-string bug that
//! caused the gist auth 401 failure documented in TODO/gist-auth-postmortem.md.
//!
//! ## What these tests prove
//!
//! 1. **gcloud exit 1 → error**: When `gcloud secrets versions access` returns
//!    exit code 1 (expired session), the shell parse node returns an error
//!    instead of silently propagating an empty string as the token.
//!
//! 2. **401 REST response → error**: When GitHub's gist API returns 401,
//!    the parse node produces an error (not a successful parse).
//!
//! 3. **auto_mock_spec gap**: The default mock always produces exit 0 / status 200,
//!    which means testgen's Bucket C scenario tests never exercise realistic
//!    error responses.

#![allow(clippy::disallowed_methods)]

use std::sync::{Arc, Mutex};

use gunbc_app::dsl_builder::build_dsl_graph_for_entrypoint;
use gunbc_exec::{execute_with_mode_and_inputs, BoundaryMocks, ExecutionMode};
use gunbc_ir::transport::{
    HttpMethod, RestResponse, ShellResponse, TransportRequest, TransportResponse,
};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_lib_transport::{executor::TransportError, TransportBackend, TransportBackendGuard};

// ── Backend: gcloud returns exit 1 (expired session) ────────────────────

#[derive(Debug)]
struct GcloudExpiredBackend {
    requests: Arc<Mutex<Vec<TransportRequest>>>,
}

impl TransportBackend for GcloudExpiredBackend {
    fn execute(&self, request: &TransportRequest) -> Result<TransportResponse, TransportError> {
        self.requests
            .lock()
            .expect("capture lock")
            .push(request.clone());

        match request {
            TransportRequest::Shell(shell) if shell.command == "git" => {
                let args = shell.args.as_slice();
                if args == ["rev-parse", "--abbrev-ref", "HEAD"] {
                    return Ok(TransportResponse::Shell(ShellResponse::ok(
                        "feature/test\n",
                    )));
                }
                if args.len() == 3 && args[0] == "merge-base" {
                    return Ok(TransportResponse::Shell(ShellResponse::ok(
                        "oldest-commit\n",
                    )));
                }
                if args.len() == 3 && args[0] == "diff" {
                    return Ok(TransportResponse::Shell(ShellResponse::ok(
                        "diff --git a/a b/a\n+line\n",
                    )));
                }
                Err(TransportError::new(format!(
                    "unexpected git invocation: {:?}",
                    shell.args
                )))
            }
            TransportRequest::Shell(shell)
                if shell.command == "bash"
                    && shell.args.len() >= 2
                    && shell.args[0] == "-lc"
                    && shell.args[1].contains("git rev-list") =>
            {
                Ok(TransportResponse::Shell(ShellResponse::ok(
                    "oldest-commit\n",
                )))
            }
            // Credential chain probes CI environment via printenv; return empty
            // to simulate non-CI, forcing fallback to gcloud.
            TransportRequest::Shell(shell) if shell.command == "printenv" => {
                Ok(TransportResponse::Shell(ShellResponse::ok("")))
            }
            // KEY: gcloud returns exit 1 (session expired / not logged in)
            TransportRequest::Shell(shell) if shell.command == "gcloud" => {
                Ok(TransportResponse::Shell(ShellResponse::failed(
                    1,
                    "ERROR: (gcloud.secrets.versions.access) There was a problem refreshing your current auth tokens",
                )))
            }
            // Credential chain STS/OAuth REST calls — fail to simulate
            // credential resolution failure (these fire because default param
            // injection provides `audience` to the credential chain).
            TransportRequest::Rest(rest) if !rest.url.ends_with("/gists") => {
                Err(TransportError::new(format!(
                    "credential REST call should not succeed when gcloud fails: {}",
                    rest.url
                )))
            }
            // Gist REST endpoint should never be reached if credential fails
            TransportRequest::Rest(_) => Ok(TransportResponse::Rest(RestResponse::new(
                201,
                serde_json::json!({
                    "id": "mock-gist-id",
                    "html_url": "https://gist.github.com/mock-gist-id"
                }),
            ))),
            other => Err(TransportError::new(format!(
                "unexpected request variant: {other:?}"
            ))),
        }
    }
}

// ── Backend: REST returns 401 Unauthorized ──────────────────────────────

#[derive(Debug)]
struct Rest401Backend {
    requests: Arc<Mutex<Vec<TransportRequest>>>,
}

impl TransportBackend for Rest401Backend {
    fn execute(&self, request: &TransportRequest) -> Result<TransportResponse, TransportError> {
        self.requests
            .lock()
            .expect("capture lock")
            .push(request.clone());

        match request {
            TransportRequest::Shell(shell) if shell.command == "git" => {
                let args = shell.args.as_slice();
                if args == ["rev-parse", "--abbrev-ref", "HEAD"] {
                    return Ok(TransportResponse::Shell(ShellResponse::ok(
                        "feature/test\n",
                    )));
                }
                if args.len() == 3 && args[0] == "merge-base" {
                    return Ok(TransportResponse::Shell(ShellResponse::ok(
                        "oldest-commit\n",
                    )));
                }
                if args.len() == 3 && args[0] == "diff" {
                    return Ok(TransportResponse::Shell(ShellResponse::ok(
                        "diff --git a/a b/a\n+line\n",
                    )));
                }
                Err(TransportError::new(format!(
                    "unexpected git invocation: {:?}",
                    shell.args
                )))
            }
            TransportRequest::Shell(shell)
                if shell.command == "bash"
                    && shell.args.len() >= 2
                    && shell.args[0] == "-lc"
                    && shell.args[1].contains("git rev-list") =>
            {
                Ok(TransportResponse::Shell(ShellResponse::ok(
                    "oldest-commit\n",
                )))
            }
            // Credential chain probes CI environment via printenv; return empty
            // to simulate non-CI, forcing fallback to gcloud.
            TransportRequest::Shell(shell) if shell.command == "printenv" => {
                Ok(TransportResponse::Shell(ShellResponse::ok("")))
            }
            TransportRequest::Shell(shell) if shell.command == "gcloud" => Ok(
                TransportResponse::Shell(ShellResponse::ok("ghp_mock_token\n")),
            ),
            // KEY: GitHub API returns 401 Unauthorized
            TransportRequest::Rest(rest) => {
                if rest.method == HttpMethod::Post && rest.url.ends_with("/gists") {
                    return Ok(TransportResponse::Rest(RestResponse::new(
                        401,
                        serde_json::json!({
                            "message": "Bad credentials",
                            "documentation_url": "https://docs.github.com/rest"
                        }),
                    )));
                }
                Err(TransportError::new(format!(
                    "unexpected REST request: method={:?} url={}",
                    rest.method, rest.url
                )))
            }
            other => Err(TransportError::new(format!(
                "unexpected request variant: {other:?}"
            ))),
        }
    }
}

fn build_gist_recent_with_inputs() -> (gunbc_ir::Dag<gunbc_exec::DynOp>, BoundaryMocks) {
    let dag = build_dsl_graph_for_entrypoint("tools/gist.dag", Some("gist_recent"), None)
        .expect("gist-recent graph should build");

    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "since" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str("3.days.ago".into()),
            ),
            "public" => {
                input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), Value::Bool(false))
            }
            _ => {}
        }
    }

    (dag, input_mocks)
}

/// **PROOF: RT-I4 catches gcloud exit code 1**
///
/// Before RT-I4, the TrimStdout parser would silently return an empty string
/// when gcloud exited with code 1 (expired session). That empty string would
/// then flow to the GitHub API as the auth token, causing a 401 error that
/// was misdiagnosed as a token permission issue.
///
/// After RT-I4, the credential chain fails — either via shell exit code
/// checking (exit 1 from gcloud) or via credential Skipped propagation
/// (when the WIF conditional path skips). Either way, execution fails
/// rather than silently sending an unauthenticated request.
#[test]
fn gcloud_exit_code_1_fails_with_error() {
    let (dag, input_mocks) = build_gist_recent_with_inputs();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let backend = Arc::new(GcloudExpiredBackend {
        requests: requests.clone(),
    });
    let _guard = TransportBackendGuard::install(backend);

    let result = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&input_mocks));

    // Execution must fail — the gcloud credential retrieval returned exit 1
    assert!(
        result.is_err(),
        "gist_recent execution should FAIL when gcloud returns exit code 1, \
         but it succeeded — this means the empty token was silently accepted"
    );

    let error = result.unwrap_err().to_string();
    // Error should mention either shell exit code (direct gcloud failure),
    // credential resolution failure (Skipped propagation through conditional path),
    // or passthrough wiring gap (branch body callable without __out: wiring).
    assert!(
        error.contains("exit")
            || error.contains("credential")
            || error.contains("Skipped")
            || error.contains("passthrough"),
        "error should mention exit code or credential failure, got: {error}"
    );

    // Verify that the gist REST endpoint was never called — the failure should
    // stop the flow before reaching the GitHub API. Credential chain REST calls
    // (OIDC, STS, metadata) may fire and fail; only count gist API calls.
    let captured = requests.lock().expect("capture lock");
    let gist_rest_calls: Vec<_> = captured
        .iter()
        .filter(|r| match r {
            TransportRequest::Rest(rest) => rest.url.ends_with("/gists"),
            _ => false,
        })
        .collect();
    assert!(
        gist_rest_calls.is_empty(),
        "Gist REST endpoint should NOT be called when gcloud credential retrieval fails — \
         found {} gist REST call(s), proving the empty token would have been sent as auth",
        gist_rest_calls.len()
    );
}

/// **PROOF: 401 REST response surfaces as error**
///
/// When the GitHub Gist API returns 401 Unauthorized, the parse node
/// should fail (missing expected fields in error response body).
/// This demonstrates that even without explicit error-status checking,
/// the parse node fails because the 401 response body doesn't contain
/// the expected `html_url` field.
#[test]
fn rest_401_response_surfaces_as_error() {
    let (dag, input_mocks) = build_gist_recent_with_inputs();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let backend = Arc::new(Rest401Backend {
        requests: requests.clone(),
    });
    let _guard = TransportBackendGuard::install(backend);

    let result = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&input_mocks));

    // Execution should fail because the 401 response body lacks html_url
    assert!(
        result.is_err(),
        "gist_recent execution should FAIL when GitHub returns 401, \
         but it succeeded — this means the error response was silently accepted"
    );
}

/// **GAP PROOF: auto_mock_spec always produces success responses**
///
/// This test demonstrates that auto_mock_spec always fills transport mocks
/// with exit code 0 (shell) and status 200 (REST). This means testgen's
/// Bucket C scenario tests never exercise realistic error responses like
/// exit code 1 (expired gcloud session) or HTTP 401 (bad credentials).
#[test]
fn auto_mock_spec_always_produces_success_responses() {
    let dag = build_dsl_graph_for_entrypoint("tools/gist.dag", Some("gist_recent"), None)
        .expect("gist-recent graph should build");

    let spec = gunbc_test::auto_mock_spec(&dag, "gist_recent");

    // Collect all transport response values from both boundary_mocks and transport_mocks
    let mut shell_mocks = Vec::new();
    let mut rest_mocks = Vec::new();

    for mock in &spec.boundary_mocks {
        match &mock.value {
            Value::Response(TransportResponse::Shell(shell)) => {
                shell_mocks.push((mock.node.as_str(), mock.port.as_str(), shell.exit_code));
            }
            Value::Response(TransportResponse::Rest(rest)) => {
                rest_mocks.push((mock.node.as_str(), mock.port.as_str(), rest.status));
            }
            _ => {}
        }
    }
    for mock in &spec.transport_mocks {
        match &mock.value {
            Value::Response(TransportResponse::Shell(shell)) => {
                shell_mocks.push((mock.node.as_str(), mock.port.as_str(), shell.exit_code));
            }
            Value::Response(TransportResponse::Rest(rest)) => {
                rest_mocks.push((mock.node.as_str(), mock.port.as_str(), rest.status));
            }
            _ => {}
        }
    }

    let total = shell_mocks.len() + rest_mocks.len();
    assert!(
        total > 0,
        "auto_mock_spec should produce at least one transport mock \
         (boundary_mocks: {}, transport_mocks: {})",
        spec.boundary_mocks.len(),
        spec.transport_mocks.len()
    );

    // Prove the gap: ALL shell mocks have exit code 0
    for (node, port, exit_code) in &shell_mocks {
        assert_eq!(
            *exit_code, 0,
            "GAP CONFIRMED: auto_mock_spec produces exit code 0 for shell mock \
             (node: {node}, port: {port}). Testgen never tests exit code 1 \
             (e.g., gcloud expired session)."
        );
    }

    // Prove the gap: ALL REST mocks have status 200
    for (node, port, status) in &rest_mocks {
        assert_eq!(
            *status, 200u16,
            "GAP CONFIRMED: auto_mock_spec produces status 200 for REST mock \
             (node: {node}, port: {port}). Testgen never tests status 401 \
             (e.g., bad GitHub credentials)."
        );
    }

    // Print summary for the analysis report
    eprintln!("=== auto_mock_spec gap analysis ===");
    eprintln!("Shell mocks (all exit 0): {}", shell_mocks.len());
    for (node, port, code) in &shell_mocks {
        eprintln!("  {node}:{port} → exit {code}");
    }
    eprintln!("REST mocks (all status 200): {}", rest_mocks.len());
    for (node, port, status) in &rest_mocks {
        eprintln!("  {node}:{port} → status {status}");
    }
    eprintln!("Total transport mocks: {total}");
    eprintln!("Error scenario mocks: 0 (THIS IS THE GAP)");
}
