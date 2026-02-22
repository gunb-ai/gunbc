//! Shared helpers for `graph_mock.rs` fixtures.
//!
//! These constructors centralize common mock payloads used across large tool
//! mock-spec files (llm-ops, gist, review) so they stay consistent.

use gunbc_ir::transport::cloud::{
    CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef,
};
use gunbc_ir::{SecretString, Value};
use std::collections::BTreeMap;

/// Build a standard bearer credential mock payload.
pub fn mock_bearer_credential(token: &str) -> Value {
    let mut map = BTreeMap::new();
    map.insert(
        "token".to_string(),
        Value::Secret(SecretString::new(token.to_string())),
    );
    map.insert("source_type".to_string(), Value::Str("static".to_string()));
    map.insert("scheme".to_string(), Value::Str("bearer".to_string()));
    map.insert(
        "cap".to_string(),
        Value::Secret(SecretString::new("capability")),
    );
    Value::Map(map)
}

/// Build the default local-dev GCP cloud secret config used by tests.
pub fn mock_gcp_local_cloud_config() -> Value {
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
