//! Mock specs for GCP WIF + Secret Manager graphs.

use gunbc_ir::transport::rest::RestResponse;
use gunbc_ir::transport::TransportResponse;
use gunbc_ir::{AuthScheme, Credential, Secret, Value};
use gunbc_test::MockSpec;

fn mock_credential() -> Value {
    let cred = Credential::new(Secret::static_value("mock-secret"), AuthScheme::Bearer);
    cred.into()
}

/// Mock spec for GCP GitHub Actions WIF + Secret Manager.
#[gunbc_testgen_registry_macros::testgen_target(
    name = "gcp-wif-secret-github",
    output = "lib/gcp-ops/src/generated_tests.rs",
    module = "gcp_wif_secret_generated_tests",
    builder = "crate::graph::build_gcp_secret_manager_credential_graph_github()",
    no_boundary_tests
)]
pub fn gcp_github_mock_spec() -> MockSpec {
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
        "payload": {
            "data": "bW9jay1zZWNyZXQ="
        }
    }));

    MockSpec::new("gcp-wif-secret-github")
        .input_mock(
            "prepare_github_oidc",
            "audience",
            Value::Str(
                "projects/123/locations/global/workloadIdentityPools/github/providers/gha"
                    .into(),
            ),
        )
        .input_mock(
            "prepare_github_oidc",
            "request_url",
            Value::Str("https://example.com/oidc".into()),
        )
        .input_mock(
            "prepare_github_oidc",
            "request_token",
            Value::Str("mock-oidc-request-token".into()),
        )
        .input_mock(
            "prepare_sts",
            "audience",
            Value::Str(
                "projects/123/locations/global/workloadIdentityPools/github/providers/gha"
                    .into(),
            ),
        )
        .input_mock(
            "prepare_impersonate",
            "service_account",
            Value::Str("ci-secrets@mock.iam.gserviceaccount.com".into()),
        )
        .input_mock(
            "prepare_secret_access",
            "project",
            Value::Str("mock-secrets".into()),
        )
        .input_mock(
            "prepare_secret_access",
            "secret",
            Value::Str("github".into()),
        )
        .input_mock(
            "build_credential",
            "scheme",
            Value::Str("bearer".into()),
        )
        .input_mock(
            "build_credential",
            "source_id",
            Value::Str("github".into()),
        )
        .transport_mock(
            "execute_github_oidc",
            "response",
            Value::Response(TransportResponse::Rest(oidc_response)),
        )
        .transport_mock(
            "execute_sts",
            "response",
            Value::Response(TransportResponse::Rest(sts_response)),
        )
        .transport_mock(
            "execute_impersonate",
            "response",
            Value::Response(TransportResponse::Rest(impersonate_response)),
        )
        .transport_mock(
            "execute_secret_access",
            "response",
            Value::Response(TransportResponse::Rest(secret_response)),
        )
        .boundary("build_credential", "credential", mock_credential())
}
