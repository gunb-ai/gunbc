//! Azure OIDC + Key Vault ops (stub).
//!
//! Placeholder implementation to keep provider-neutral modeling honest.

mod ops;
mod graph;

pub use graph::{build_azure_key_vault_credential_graph, AzureKeyVaultGraphOp};
pub use ops::AzureOps;
