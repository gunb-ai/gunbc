//! Stub graph for Azure Key Vault.

use crate::ops::AzureOps;
use gunbc_delegate_macros::DelegateExecutable;
use gunbc_ir::build::{list, optional, port};
use gunbc_ir::{BuilderError, Dag, DagBuilder, Node};

#[derive(Debug, Clone, DelegateExecutable)]
pub enum AzureKeyVaultGraphOp {
    Azure(AzureOps),
}

/// Placeholder DAG for Azure Key Vault credentials.
pub fn build_azure_key_vault_credential_graph() -> Result<Dag<AzureKeyVaultGraphOp>, BuilderError> {
    let mut builder: DagBuilder<AzureKeyVaultGraphOp> = DagBuilder::new();

    builder.add_root_node(Node::opaque(
        "azure_key_vault_stub",
        vec![
            port("config", "CloudSecretConfig"),
            port("scheme", "String"),
            optional("header_name", "OptionalString"),
            port("source_id", "String"),
            list("required_scopes", "String"),
            optional("lifetime_seconds", "OptionalInt"),
            optional("request_url", "OptionalString"),
            optional("request_token", "OptionalString"),
        ],
        vec![port("credential", "Credential")],
        AzureKeyVaultGraphOp::Azure(AzureOps::Unsupported),
    ))?;

    Ok(builder.build())
}

/// Placeholder DAG for Azure Key Vault secret upsert.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "azure-keyvault-upsert-stub",
    builder = "build_azure_key_vault_upsert_graph()",
    returns_result
)]
pub fn build_azure_key_vault_upsert_graph() -> Result<Dag<AzureKeyVaultGraphOp>, BuilderError> {
    let mut builder: DagBuilder<AzureKeyVaultGraphOp> = DagBuilder::new();

    builder.add_root_node(Node::opaque(
        "azure_key_vault_upsert_stub",
        vec![
            port("config", "CloudSecretConfig"),
            port("secret_value", "Secret"),
        ],
        vec![port("version", "String")],
        AzureKeyVaultGraphOp::Azure(AzureOps::Unsupported),
    ))?;

    Ok(builder.build())
}
