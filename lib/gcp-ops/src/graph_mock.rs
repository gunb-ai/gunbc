//! Mock specs for GCP WIF + Secret Manager graphs.

use gunbc_ir::transport::rest::RestResponse;
use gunbc_ir::transport::TransportResponse;
use gunbc_ir::{AuthScheme, Credential, Secret, SecretString, Value};
use gunbc_primitives::NetworkHandle;
use gunbc_test::{MockSpec, NodeExample, OutputMatcher};

fn mock_credential() -> Value {
    let cred = Credential::new(Secret::static_value("mock-secret"), AuthScheme::Bearer);
    cred.into()
}

fn mock_net_handle() -> Value {
    NetworkHandle.into()
}

/// Mock spec for GCP GitHub Actions WIF + Secret Manager.
#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "gcp-wif-secret-github",
    builder = "crate::graph::build_gcp_secret_manager_credential_graph_github()"
)]
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
                "projects/123/locations/global/workloadIdentityPools/github/providers/gha".into(),
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
                "projects/123/locations/global/workloadIdentityPools/github/providers/gha".into(),
            ),
        )
        .input_mock(
            "prepare_impersonate",
            "service_account",
            Value::Str("ci-secrets@mock.iam.gserviceaccount.com".into()),
        )
        .input_mock(
            "parse_impersonate",
            "response",
            Value::Response(TransportResponse::Rest(impersonate_response.clone())),
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
        .input_mock("build_credential", "scheme", Value::Str("bearer".into()))
        .input_mock("build_credential", "source_id", Value::Str("github".into()))
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
        .boundary("net_env", "net", mock_net_handle())
        .node_example(
            NodeExample::new("build_credential")
                .input("secret", Value::Str("mock-secret".into()))
                .input("scheme", Value::Str("bearer".into()))
                .input("source_id", Value::Str("github".into()))
                .output("credential", OutputMatcher::Any)
                .description("Builds a bearer credential from secret material"),
        )
        // Pure nodes — skip example enforcement for now
        .skip_node_example("prepare_github_oidc")
        .skip_node_example("parse_github_oidc")
        .skip_node_example("prepare_sts")
        .skip_node_example("parse_sts")
        .skip_node_example("should_impersonate")
        .skip_node_example("prepare_impersonate")
        .skip_node_example("parse_impersonate")
        .skip_node_example("prepare_secret_access")
        .skip_node_example("parse_secret_access")
        .skip_node_example("net_env")
}

/// Mock spec for local-dev ADC auth + Secret Manager credential flow.
///
/// The local auth phase is a `local_auth_upsert` sub-DAG that:
/// 1. Checks if the ADC file exists (File transport)
/// 2. Reads the ADC file (File transport)
/// 3. Try-refreshes OAuth2 token (REST transport)
/// 4. If refresh fails with auth error: runs gcloud auth (Shell transport)
/// 5. Re-reads ADC, retries OAuth2 refresh, merges result
///
/// This is a composition helper — it gets included in higher-level tool
/// mock specs via `include_prefixed_runtime_mocks`.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn gcp_local_mock_spec() -> MockSpec {
    use gunbc_ir::transport::file::FileResponse;

    let adc_path = crate::ops::adc_file_path();

    // Mock ADC file content (minimal authorized_user credentials)
    let mock_adc_json = serde_json::json!({
        "type": "authorized_user",
        "client_id": "mock-client-id.apps.googleusercontent.com",
        "client_secret": "mock-client-secret",
        "refresh_token": "mock-refresh-token"
    });

    let impersonate_response = RestResponse::ok(serde_json::json!({
        "accessToken": "mock-sa-token",
        "expireTime": "2025-01-01T00:00:00Z"
    }));
    let secret_response = RestResponse::ok(serde_json::json!({
        "payload": {
            "data": "bW9jay1zZWNyZXQ="
        }
    }));
    let oauth2_response = RestResponse::ok(serde_json::json!({
        "access_token": "mock-local-token",
        "expires_in": 3599,
        "token_type": "Bearer"
    }));

    MockSpec::new("gcp-wif-secret-local")
        // local_auth_upsert sub-DAG boundaries: ADC file check (File transport)
        .boundary(
            "local_auth_upsert/execute_check",
            "response",
            Value::Response(TransportResponse::File(
                FileResponse::exists_result(&adc_path, true),
            )),
        )
        // local_auth_upsert sub-DAG: read ADC file (File transport)
        .boundary(
            "local_auth_upsert/execute_read_adc",
            "response",
            Value::Response(TransportResponse::File(
                FileResponse::read_ok(&adc_path, mock_adc_json.to_string()),
            )),
        )
        // local_auth_upsert sub-DAG: OAuth2 token try-refresh (REST transport)
        .boundary(
            "local_auth_upsert/execute_oauth2",
            "response",
            Value::Response(TransportResponse::Rest(oauth2_response)),
        )
        // Re-auth branch boundaries (skipped in happy path — mock as Skipped)
        .boundary(
            "local_auth_upsert/execute_gcloud_auth",
            "response",
            Value::Skipped,
        )
        .boundary(
            "local_auth_upsert/execute_reread_adc",
            "response",
            Value::Skipped,
        )
        .boundary(
            "local_auth_upsert/execute_retry_oauth2",
            "response",
            Value::Skipped,
        )
        .boundary("local_auth_upsert/net_env", "net", mock_net_handle())
        // IAM ensure (local dev only) — REST-based check + conditional set
        .input_mock(
            "prepare_ensure_iam",
            "project",
            Value::Str("mock-secrets".into()),
        )
        .input_mock(
            "prepare_ensure_iam",
            "service_account",
            Value::Str("ci-secrets@mock.iam.gserviceaccount.com".into()),
        )
        .transport_mock(
            "execute_get_iam",
            "response",
            Value::Response(TransportResponse::Rest(RestResponse::ok(
                serde_json::json!({
                    "bindings": [{
                        "role": "roles/secretmanager.secretAccessor",
                        "members": ["serviceAccount:ci-secrets@mock.iam.gserviceaccount.com"]
                    }],
                    "etag": "mock-etag"
                }),
            ))),
        )
        // setIamPolicy is skipped (binding already exists in mock)
        .transport_mock(
            "execute_set_iam",
            "response",
            Value::Skipped,
        )
        .input_mock(
            "prepare_impersonate",
            "service_account",
            Value::Str("ci-secrets@mock.iam.gserviceaccount.com".into()),
        )
        .input_mock(
            "parse_impersonate",
            "response",
            Value::Response(TransportResponse::Rest(impersonate_response.clone())),
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
        .input_mock("build_credential", "scheme", Value::Str("bearer".into()))
        .input_mock("build_credential", "source_id", Value::Str("github".into()))
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
        .boundary("net_env", "net", mock_net_handle())
        .node_example(
            NodeExample::new("build_credential")
                .input("secret", Value::Str("mock-secret".into()))
                .input("scheme", Value::Str("bearer".into()))
                .input("source_id", Value::Str("github".into()))
                .output("credential", OutputMatcher::Any)
                .description("Builds a bearer credential from secret material"),
        )
        // Sub-DAG internal nodes — tested at their own level
        .skip_node_example("local_auth_upsert")
        .skip_node_example("should_impersonate")
        .skip_node_example("prepare_ensure_iam")
        .skip_node_example("check_iam_binding")
        .skip_node_example("parse_set_iam")
        .skip_node_example("prepare_impersonate")
        .skip_node_example("parse_impersonate")
        .skip_node_example("prepare_secret_access")
        .skip_node_example("parse_secret_access")
        .skip_node_example("net_env")
}

/// Mock spec for GCP GitHub Actions WIF + Secret Manager upsert.
#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "gcp-wif-secret-upsert-github",
    builder = "crate::graph::build_gcp_secret_manager_upsert_graph_github()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "gcp-wif-secret-upsert-github",
    output = "lib/gcp-ops/src/generated_tests_upsert.rs",
    module = "gcp_wif_secret_upsert_generated_tests",
    builder = "crate::graph::build_gcp_secret_manager_upsert_graph_github()",
    no_boundary_tests
)]
pub fn gcp_github_upsert_mock_spec() -> MockSpec {
    let oidc_response = RestResponse::ok(serde_json::json!({"value": "mock-oidc-token"}));
    let sts_response = RestResponse::ok(serde_json::json!({
        "access_token": "mock-sts-token",
        "expires_in": 3600
    }));
    let impersonate_response = RestResponse::ok(serde_json::json!({
        "accessToken": "mock-sa-token",
        "expireTime": "2025-01-01T00:00:00Z"
    }));
    let secret_get_response = RestResponse::new(404, serde_json::json!({"error": "not found"}));
    let secret_create_response =
        RestResponse::ok(serde_json::json!({"name": "projects/mock/secrets/github"}));
    let add_version_response = RestResponse::ok(serde_json::json!({
        "name": "projects/mock/secrets/github/versions/1"
    }));

    MockSpec::new("gcp-wif-secret-upsert-github")
        .input_mock(
            "prepare_github_oidc",
            "audience",
            Value::Str(
                "projects/123/locations/global/workloadIdentityPools/github/providers/gha".into(),
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
                "projects/123/locations/global/workloadIdentityPools/github/providers/gha".into(),
            ),
        )
        .input_mock(
            "prepare_impersonate",
            "service_account",
            Value::Str("ci-secrets@mock.iam.gserviceaccount.com".into()),
        )
        .input_mock(
            "parse_impersonate",
            "response",
            Value::Response(TransportResponse::Rest(impersonate_response.clone())),
        )
        .input_mock(
            "prepare_secret_get",
            "project",
            Value::Str("mock-secrets".into()),
        )
        .input_mock("prepare_secret_get", "secret", Value::Str("github".into()))
        .input_mock(
            "prepare_secret_create",
            "project",
            Value::Str("mock-secrets".into()),
        )
        .input_mock(
            "prepare_secret_create",
            "secret",
            Value::Str("github".into()),
        )
        .input_mock(
            "prepare_secret_add_version",
            "project",
            Value::Str("mock-secrets".into()),
        )
        .input_mock(
            "prepare_secret_add_version",
            "secret",
            Value::Str("github".into()),
        )
        .input_mock(
            "prepare_secret_add_version",
            "secret_value",
            Value::Secret(SecretString::new("mock-secret-value")),
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
            "execute_secret_get",
            "response",
            Value::Response(TransportResponse::Rest(secret_get_response)),
        )
        .transport_mock(
            "execute_secret_create",
            "response",
            Value::Response(TransportResponse::Rest(secret_create_response)),
        )
        .transport_mock("execute_secret_create", "skip", Value::Bool(false))
        .transport_mock(
            "execute_secret_add_version",
            "response",
            Value::Response(TransportResponse::Rest(add_version_response)),
        )
        .boundary("net_env", "net", mock_net_handle())
        // Pure nodes — skip example enforcement for now
        .skip_node_example("prepare_github_oidc")
        .skip_node_example("parse_github_oidc")
        .skip_node_example("prepare_sts")
        .skip_node_example("parse_sts")
        .skip_node_example("should_impersonate")
        .skip_node_example("prepare_impersonate")
        .skip_node_example("parse_impersonate")
        .skip_node_example("prepare_secret_get")
        .skip_node_example("parse_secret_get")
        .skip_node_example("prepare_secret_create")
        .skip_node_example("parse_secret_create")
        .skip_node_example("prepare_secret_add_version")
        .skip_node_example("parse_secret_add_version")
        .skip_node_example("net_env")
}
