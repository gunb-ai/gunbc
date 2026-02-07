//! Environment node pattern for resource acquisition at DAG boundaries.
//!
//! Environment nodes:
//! - have no inputs (root nodes)
//! - perform acquisition at the boundary
//! - emit typed resources on output ports
//! - provide mock outputs for DryRun/testgen

use crate::{ExecError, Executable, OutputMap};
use gunbc_ir::Value;
use std::collections::HashMap;
use std::fmt;

/// Trait for environment boundary nodes.
///
/// Environment nodes are zero-input ops that acquire system resources and emit
/// them as typed outputs. Implementors provide real outputs for execution and
/// mock outputs for DryRun/testgen.
pub trait EnvNode: fmt::Debug {
    /// Acquire real outputs (boundary acquisition).
    fn env_outputs(&self) -> Result<HashMap<String, Value>, ExecError>;

    /// Mock outputs for DryRun/testgen.
    fn mock_outputs(&self) -> HashMap<String, Value>;
}

impl<T: EnvNode> Executable for T {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        self.env_outputs()
    }
}

/// Helper for single-output environment nodes.
pub fn single_output(port: &'static str, value: impl Into<Value>) -> HashMap<String, Value> {
    OutputMap::new().value(port, value.into()).build()
}
