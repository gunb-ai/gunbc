//! Stub ops for Azure Key Vault-backed credentials.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AzureOps {
    /// Placeholder: Azure Key Vault support is not yet implemented.
    Unsupported,
}

impl Executable for AzureOps {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        Err(ExecError::new(
            "Azure Key Vault support is stubbed (not implemented)",
        ))
    }
}
