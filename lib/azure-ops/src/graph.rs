//! Stub graph for Azure Key Vault.

use crate::ops::AzureOps;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::build::port;
use gunbc_ir::{Dag, DagBuilder, Node, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AzureKeyVaultGraphOp {
    Azure(AzureOps),
}

impl Executable for AzureKeyVaultGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            AzureKeyVaultGraphOp::Azure(op) => op.execute(inputs),
        }
    }
}

/// Placeholder DAG for Azure Key Vault credentials.
pub fn build_azure_key_vault_credential_graph() -> Dag<AzureKeyVaultGraphOp> {
    let mut builder: DagBuilder<AzureKeyVaultGraphOp> = DagBuilder::new();

    builder
        .add_root_node(Node::opaque(
            "azure_key_vault_stub",
            vec![],
            vec![port("credential", "Credential")],
            AzureKeyVaultGraphOp::Azure(AzureOps::Unsupported),
        ))
        .expect("azure_key_vault_stub node");

    builder.build()
}
