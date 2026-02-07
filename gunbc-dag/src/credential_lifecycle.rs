//! Credential lifecycle testgen targets.

use gunbc_ir::transport::rest::RestResponse;
use gunbc_ir::{AuthScheme, Credential, Secret, Value};
use gunbc_test::{MockSpec, NodeExample, OutputMatcher};

fn mock_credential() -> Value {
    let cred = Credential::new(Secret::static_value("ghp_mock_token"), AuthScheme::Bearer);
    cred.into()
}

/// Mock specification for GitHub credential lifecycle testing.
#[gunbc_testgen_registry_macros::testgen_target(
    name = "github-credential-lifecycle",
    output = "gunbc-dag/src/generated_tests_credential_github.rs",
    module = "github_credential_lifecycle_generated_tests",
    builder = "gunbc_lib_transport::credential_graph::build_github_credential_graph()",
    no_boundary_tests
)]
pub fn github_credential_lifecycle_mock_spec() -> MockSpec {
    let response = RestResponse::ok(serde_json::json!({"resources": {}}));

    MockSpec::new("github-credential-lifecycle")
        // Env: resolved auth token
        .boundary("credential_env", "credential:github", mock_credential())
        // Transport mock
        .transport_mock(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response.clone())),
        )
        .boundary(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        // Parse outputs (status + ok)
        .boundary("parse_status", "status", Value::Int(200))
        .boundary("parse_status", "ok", Value::Bool(true))
        // Credential resource: basic (acquire + timeout)
        .resource_credential("credential:github", Some(3_600_000))
        // Credential resource: refreshable (acquire + timeout + refresh)
        .resource_credential_refreshable("credential:github:refreshable", 3_600_000, 3_600_000)
        // Node I/O examples
        .node_example(
            NodeExample::new("resolve_auth")
                .output("service", OutputMatcher::exact(Value::Str("github".into())))
                .output(
                    "env_var",
                    OutputMatcher::exact(Value::Str("GITHUB_TOKEN".into())),
                )
                .output(
                    "scheme",
                    OutputMatcher::exact(Value::Str("bearer".into())),
                )
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str(String::new())),
                )
                .description("Resolve auth maps GitHub to GITHUB_TOKEN bearer auth"),
        )
        .node_example(
            NodeExample::new("prepare_request")
                .output("request", OutputMatcher::non_empty())
                .output("skip", OutputMatcher::exact(Value::Bool(false)))
                .description("Prepare request builds GitHub REST rate_limit call"),
        )
        .node_example(
            NodeExample::new("parse_status")
                .input(
                    "response",
                    Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                        RestResponse::ok(serde_json::json!({"resources": {}})),
                    )),
                )
                .output("status", OutputMatcher::exact(Value::Int(200)))
                .output("ok", OutputMatcher::exact(Value::Bool(true)))
                .description("Parse status extracts HTTP status and ok flag"),
        )
        .skip_node_example("credential_env")
}

// Generated tests (from `make testgen`)
#[cfg(test)]
mod generated_tests {
    include!("generated_tests_credential_github.rs");
}
