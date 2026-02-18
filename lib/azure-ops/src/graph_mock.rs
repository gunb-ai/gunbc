//! Mock specification for Azure Key Vault (stub).

use gunbc_ir::transport::cloud::{
    CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef,
};
use gunbc_ir::{AuthScheme, Credential, Secret, Value};
use gunbc_test::MockSpec;

fn mock_config() -> CloudSecretConfig {
    CloudSecretConfig {
        provider: CloudProviderKind::Azure,
        runtime: CloudRuntimeKind::GitHubActions,
        audience: "azure-audience".to_string(),
        project_or_account: "azure-tenant".to_string(),
        secret: CloudSecretRef {
            prefix: "ci-".to_string(),
            name: "anthropic".to_string(),
            delimiter: "".to_string(),
            version: None,
        },
        service_account_or_role: Some("azure-client-id".to_string()),
        impersonate_account_or_role: None,
    }
}

fn mock_credential() -> Value {
    let cred = Credential::new(Secret::static_value("mock-token"), AuthScheme::Bearer);
    cred.into()
}

/// Mock spec for Azure Key Vault stub graph.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "azure-keyvault-stub",
    builder = "crate::graph::build_azure_key_vault_credential_graph()",
    returns_result
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "azure-keyvault-stub",
    output = "lib/azure-ops/src/generated_tests.rs",
    module = "azure_keyvault_generated_tests",
    builder = "crate::graph::build_azure_key_vault_credential_graph().expect(\"azure stub graph should build\")",
    no_boundary_tests
)]
pub fn azure_stub_mock_spec() -> MockSpec {
    MockSpec::new("azure-keyvault-stub")
        .input_mock("azure_key_vault_stub", "config", mock_config().into())
        .input_mock(
            "azure_key_vault_stub",
            "scheme",
            Value::Str("bearer".into()),
        )
        .input_mock(
            "azure_key_vault_stub",
            "header_name",
            Value::Str(String::new()),
        )
        .input_mock(
            "azure_key_vault_stub",
            "source_id",
            Value::Str("azure".into()),
        )
        .input_mock(
            "azure_key_vault_stub",
            "request_url",
            Value::Str("https://example.com/oidc".into()),
        )
        .input_mock(
            "azure_key_vault_stub",
            "request_token",
            Value::Str("mock-oidc-token".into()),
        )
        .boundary("azure_key_vault_stub", "credential", mock_credential())
        .skip_node_example("azure_key_vault_stub")
}
