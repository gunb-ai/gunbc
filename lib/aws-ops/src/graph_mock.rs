//! Mock specification for AWS Secrets Manager (stub).

use gunbc_ir::transport::cloud::{CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef};
use gunbc_ir::{AuthScheme, Credential, Secret, Value};
use gunbc_test::MockSpec;

fn mock_config() -> CloudSecretConfig {
    CloudSecretConfig {
        provider: CloudProviderKind::Aws,
        runtime: CloudRuntimeKind::GitHubActions,
        audience: "aws-audience".to_string(),
        project_or_account: "123456789012".to_string(),
        secret: CloudSecretRef {
            prefix: "ci-".to_string(),
            name: "openai".to_string(),
            delimiter: "".to_string(),
            version: None,
        },
        service_account_or_role: Some("arn:aws:iam::123456789012:role/ci".to_string()),
        impersonate_account_or_role: None,
    }
}

fn mock_credential() -> Value {
    let cred = Credential::new(Secret::static_value("mock-token"), AuthScheme::Bearer);
    cred.into()
}

/// Mock spec for AWS Secrets Manager stub graph.
#[gunbc_testgen_registry_macros::testgen_target(
    name = "aws-secrets-stub",
    output = "lib/aws-ops/src/generated_tests.rs",
    module = "aws_secrets_generated_tests",
    builder = "crate::graph::build_aws_secrets_manager_credential_graph()",
    no_boundary_tests
)]
pub fn aws_stub_mock_spec() -> MockSpec {
    MockSpec::new("aws-secrets-stub")
        .input_mock("aws_secrets_manager_stub", "config", mock_config().into())
        .input_mock("aws_secrets_manager_stub", "scheme", Value::Str("bearer".into()))
        .input_mock("aws_secrets_manager_stub", "header_name", Value::Str(String::new()))
        .input_mock("aws_secrets_manager_stub", "source_id", Value::Str("aws".into()))
        .input_mock(
            "aws_secrets_manager_stub",
            "request_url",
            Value::Str("https://example.com/oidc".into()),
        )
        .input_mock(
            "aws_secrets_manager_stub",
            "request_token",
            Value::Str("mock-oidc-token".into()),
        )
        .boundary("aws_secrets_manager_stub", "credential", mock_credential())
        .skip_node_example("aws_secrets_manager_stub")
}
