//! Credential lifecycle testgen targets.

use gunbc_ir::transport::cloud::{
    CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef,
};
use gunbc_ir::transport::rest::RestResponse;
use gunbc_ir::{AuthScheme, Credential, Secret, Value};
use gunbc_primitives::NetworkHandle;
use gunbc_test::{MockSpec, NodeExample, OutputMatcher};

fn mock_credential() -> Value {
    let cred = Credential::new(Secret::static_value("ghp_mock_token"), AuthScheme::Bearer);
    cred.into()
}

fn mock_cloud_config() -> Value {
    CloudSecretConfig {
        provider: CloudProviderKind::Gcp,
        runtime: CloudRuntimeKind::GitHubActions,
        audience: "projects/123/locations/global/workloadIdentityPools/github/providers/gha"
            .to_string(),
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
        runtime: CloudRuntimeKind::GitHubActions,
        audience: "projects/123/locations/global/workloadIdentityPools/github/providers/gha"
            .to_string(),
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
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "github-credential-lifecycle",
    output = "gunbc-dag/src/generated_tests_credential_github.rs",
    module = "github_credential_lifecycle_generated_tests",
    builder = "gunbc_lib_cloud_ops::build_github_credential_graph()",
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
    let oidc_response = RestResponse::ok(serde_json::json!({"value": "mock-oidc-token"}));
    let sts_response = RestResponse::ok(serde_json::json!({
        "access_token": "mock-sts-token",
        "expires_in": 3600
    }));
    let impersonate_response = RestResponse::ok(serde_json::json!({
        "accessToken": "mock-sa-token",
        "expireTime": "2025-01-01T00:00:00Z"
    }));
    let secret_response = RestResponse::ok(serde_json::json!({
        "payload": { "data": "bW9jay1zZWNyZXQ=" }
    }));

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
        .boundary("bind_secret", "config", mock_cloud_config_with_secret("github"))
        .boundary("cloud_credential", "credential", mock_credential())
        .boundary(
            "cloud_credential/gcp_wif_secret/net_env",
            "net",
            NetworkHandle.into(),
        )
        .transport_mock(
            "cloud_credential/gcp_wif_secret/execute_github_oidc",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                oidc_response,
            )),
        )
        .transport_mock(
            "cloud_credential/gcp_wif_secret/execute_sts",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                sts_response,
            )),
        )
        .transport_mock(
            "cloud_credential/gcp_wif_secret/execute_impersonate",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                impersonate_response,
            )),
        )
        .transport_mock(
            "cloud_credential/gcp_wif_secret/execute_secret_access",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                secret_response,
            )),
        )
        .boundary(
            "cloud_credential/gcp_wif_secret/build_credential",
            "credential",
            mock_credential(),
        )
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
        // Live expectations: should authenticate successfully
        .live_expected_output(
            "parse_status",
            "ok",
            OutputMatcher::exact(Value::Bool(true)),
        )
        .live_expected_output("parse_status", "status", OutputMatcher::IntGe(200))
        // Credential resource: basic (acquire + timeout)
        .resource_credential("credential:github", Some(3_600_000))
        // Credential resource: refreshable (acquire + timeout + refresh)
        .resource_credential_refreshable("credential:github:refreshable", 3_600_000, 3_600_000)
        // Node I/O examples
        .node_example(
            NodeExample::new("resolve_auth")
                .output("service", OutputMatcher::exact(Value::Str("github".into())))
                .output(
                    "scheme",
                    OutputMatcher::exact(Value::Str("bearer".into())),
                )
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str(String::new())),
                )
                .description("Resolve auth maps GitHub to bearer auth"),
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
        .skip_node_example("cloud_env")
        .skip_node_example("cloud_credential")
        .skip_node_example("bind_secret")
}

// Generated tests (from `make testgen`)
#[cfg(test)]
mod generated_tests {
    include!("generated_tests_credential_github.rs");
}
