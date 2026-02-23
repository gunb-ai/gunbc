//! Credential lifecycle testgen targets.

use gunbc_ir::transport::cloud::{
    CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef,
};
use gunbc_ir::transport::rest::RestResponse;
use gunbc_ir::Value;
use gunbc_test::{MockSpec, NodeExample, OutputMatcher};

fn mock_cloud_config() -> Value {
    CloudSecretConfig {
        provider: CloudProviderKind::Gcp,
        runtime: CloudRuntimeKind::LocalDev,
        audience: "local-dev".to_string(),
        project_or_account: "mock-secrets".to_string(),
        secret: CloudSecretRef {
            prefix: "ci-".to_string(),
            name: String::new(),
            delimiter: String::new(),
            version: None,
        },
        service_account_or_role: Some("ci-secrets@mock.iam.gserviceaccount.com".to_string()),
        impersonate_account_or_role: None,
    }
    .into()
}

fn mock_cloud_config_with_secret(name: &str) -> Value {
    let mut config = CloudSecretConfig {
        provider: CloudProviderKind::Gcp,
        runtime: CloudRuntimeKind::LocalDev,
        audience: "local-dev".to_string(),
        project_or_account: "mock-secrets".to_string(),
        secret: CloudSecretRef {
            prefix: "ci-".to_string(),
            name: String::new(),
            delimiter: String::new(),
            version: None,
        },
        service_account_or_role: Some("ci-secrets@mock.iam.gserviceaccount.com".to_string()),
        impersonate_account_or_role: None,
    };
    config.secret.name = name.to_string();
    config.into()
}

/// Mock specification for GitHub credential lifecycle testing.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "github-credential-lifecycle",
    builder = "gunbc_lib_cloud_ops::build_github_credential_graph()",
    returns_result
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "github-credential-lifecycle",
    output = "gunbc-dag/src/generated_tests_credential_github.rs",
    module = "github_credential_lifecycle_generated_tests",
    builder = "gunbc_lib_cloud_ops::build_github_credential_graph()",
    returns_result,
    no_boundary_tests,
    live_flow_tests,
    live_class = "integration",
    live_fermi = "M",
    live_required(
        "GCP_WIF_PROVIDER",
        "GCP_SECRETS_PROJECT",
        "GCP_SECRETS_PREFIX",
        "ACTIONS_ID_TOKEN_REQUEST_URL",
        "ACTIONS_ID_TOKEN_REQUEST_TOKEN"
    ),
    live_required_any_of("GCP_SECRETS_SA", "GCP_SECRETS_IMPERSONATE_SA")
)]
pub fn github_credential_lifecycle_mock_spec() -> MockSpec {
    let response = RestResponse::ok(serde_json::json!({"resources": {}}));
    MockSpec::new("github-credential-lifecycle")
        // Cloud env + credential
        .boundary("cloud_env", "config", mock_cloud_config())
        .boundary(
            "cloud_env",
            "request_url",
            Value::Str("https://example.com/oidc".into()),
        )
        .boundary(
            "cloud_env",
            "request_token",
            Value::Str("mock-oidc-token".into()),
        )
        .boundary(
            "bind_secret",
            "config",
            mock_cloud_config_with_secret("github"),
        )
        // resolve_config sub-node: unpacks CloudSecretConfig into individual ports
        .boundary(
            "cloud_credential/resolve_config",
            "provider",
            Value::Str("gcp".into()),
        )
        .boundary(
            "cloud_credential/resolve_config",
            "runtime",
            Value::Str("local".into()),
        )
        .boundary(
            "cloud_credential/resolve_config",
            "audience",
            Value::Str("local-dev".into()),
        )
        .boundary(
            "cloud_credential/resolve_config",
            "project_or_account",
            Value::Str("mock-secrets".into()),
        )
        .boundary(
            "cloud_credential/resolve_config",
            "secret",
            Value::Str("ci-".into()),
        )
        .boundary(
            "cloud_credential/resolve_config",
            "version",
            Value::Str(String::new()),
        )
        .boundary(
            "cloud_credential/resolve_config",
            "service_account_or_role",
            Value::Str("ci-secrets@mock.iam.gserviceaccount.com".into()),
        )
        .boundary(
            "cloud_credential/resolve_config",
            "impersonate_account_or_role",
            Value::Str(String::new()),
        )
        // Transport mock (lowered SubDag path)
        .transport_mock(
            "credential_check/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                response.clone(),
            )),
        )
        .boundary(
            "credential_check/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        // Parse outputs (lowered SubDag path)
        .boundary("credential_check/parse_status", "status", Value::Int(200))
        .boundary("credential_check/parse_status", "ok", Value::Bool(true))
        // Live expectations: should authenticate successfully
        .live_expected_output(
            "credential_check/parse_status",
            "ok",
            OutputMatcher::exact(Value::Bool(true)),
        )
        .live_expected_output(
            "credential_check/parse_status",
            "status",
            OutputMatcher::IntGe(200),
        )
        // Credential resource: basic (acquire + timeout)
        .resource_credential("credential:github", Some(3_600_000))
        // Credential resource: refreshable (acquire + timeout + refresh)
        .resource_credential_refreshable("credential:github:refreshable", 3_600_000, 3_600_000)
        // Node I/O examples
        .node_example(
            NodeExample::new("resolve_auth")
                .output("service", OutputMatcher::exact(Value::Str("github".into())))
                .output("scheme", OutputMatcher::exact(Value::Str("bearer".into())))
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str(String::new())),
                )
                .description("Resolve auth maps GitHub to bearer auth"),
        )
        .node_example(
            NodeExample::new("credential_check/prepare_request")
                .output("request", OutputMatcher::non_empty())
                .output("skip", OutputMatcher::exact(Value::Bool(false)))
                .description("Prepare request builds GitHub REST rate_limit call"),
        )
        .node_example(
            NodeExample::new("scope_preflight")
                .input(
                    "required_scopes",
                    Value::str_list(vec!["gist:write".to_string()]),
                )
                .output("scope_verified", OutputMatcher::exact(Value::Bool(true)))
                .description("Scope preflight validates required scope declarations"),
        )
        .node_example(
            NodeExample::new("credential_check/parse_status")
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
        // Probe-observer: GCP sub-DAG terminal needs chain-safe observer
        .live_expected_output(
            "cloud_credential/gcp_wif_secret/parse_set_iam",
            "ok",
            OutputMatcher::IsBool,
        )
        .skip_node_example("cloud_env")
        .skip_node_example("cloud_credential")
        .skip_node_example("bind_secret")
}
