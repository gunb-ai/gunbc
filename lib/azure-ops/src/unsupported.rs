use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};

pub type AzureKeyVaultGraphOp = DynOp;

fn dropped_provider_error(surface: &str) -> BuilderError {
    BuilderError::InternalInvariant(format!(
        "Azure provider stack is dropped in this branch; `{surface}` is intentionally unsupported"
    ))
}

pub fn build_azure_key_vault_credential_graph() -> Result<Dag<AzureKeyVaultGraphOp>, BuilderError> {
    Err(dropped_provider_error(
        "build_azure_key_vault_credential_graph",
    ))
}

pub fn build_azure_key_vault_upsert_graph() -> Result<Dag<AzureKeyVaultGraphOp>, BuilderError> {
    Err(dropped_provider_error("build_azure_key_vault_upsert_graph"))
}
