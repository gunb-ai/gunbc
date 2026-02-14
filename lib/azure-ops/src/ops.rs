//! Stub ops for Azure Key Vault-backed credentials.

use gunbc_exec::{
    optional_int_strict, optional_str_list_strict, optional_str_strict, ExecError, Executable,
    OutputMap,
};
use gunbc_ir::{SecretString, Value};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AzureOps {
    /// Placeholder: Azure Key Vault support is not yet implemented.
    /// Returns a stub credential so the DAG is structurally valid.
    Unsupported,
}

impl Executable for AzureOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        // Validate optional inputs so wrong-typed values are rejected,
        // matching the port declarations in the graph.
        optional_str_strict(&inputs, "header_name")?;
        optional_str_list_strict(&inputs, "required_scopes")?;
        optional_int_strict(&inputs, "lifetime_seconds")?;
        optional_str_strict(&inputs, "request_url")?;
        optional_str_strict(&inputs, "request_token")?;

        // Return a stub credential so the graph is structurally valid
        // and generated tests (optional-input, dryrun) can pass.
        let mut cred = BTreeMap::new();
        cred.insert(
            "token".to_string(),
            Value::Secret(SecretString::new("<AZURE_STUB_NOT_IMPLEMENTED>")),
        );
        cred.insert("source_type".to_string(), Value::Str("stub".to_string()));
        cred.insert("scheme".to_string(), Value::Str("bearer".to_string()));
        cred.insert(
            "cap".to_string(),
            Value::Secret(SecretString::new("capability")),
        );
        OutputMap::new().value("credential", Value::Map(cred)).ok()
    }
}
