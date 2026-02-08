//! Stub ops for AWS secret-backed credentials.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AwsOps {
    /// Placeholder: AWS Secrets Manager support is not yet implemented.
    Unsupported,
}

impl Executable for AwsOps {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        Err(ExecError::new(
            "AWS Secrets Manager support is stubbed (not implemented)",
        ))
    }
}
