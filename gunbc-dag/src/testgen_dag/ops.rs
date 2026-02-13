//! Testgen DAG operations.
//!
//! All I/O happens through explicit `TransportOps::Execute` nodes in the DAG.
//! The ops here are PURE (no I/O) - they generate test code.

use gunbc_codegen::TestgenTargetDef;
use gunbc_exec::{ExecError, Executable, OutputMap};
use gunbc_ir::Value;
use std::collections::HashMap;
use std::fmt;

/// Operations for the testgen DAG.
///
/// All operations are PURE - no I/O. I/O happens via TransportOps::Execute nodes.
#[derive(Clone)]
pub enum TestgenOp {
    /// Generate test code for a target.
    Generate {
        name: String,
        target_def: TestgenTargetDef,
        generate_fn: fn(&TestgenTargetDef) -> String,
    },
}

impl fmt::Debug for TestgenOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestgenOp::Generate { name, .. } => {
                f.debug_struct("Generate").field("name", name).finish()
            }
        }
    }
}

impl Executable for TestgenOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            TestgenOp::Generate {
                name,
                target_def,
                generate_fn,
            } => {
                let def = target_def.clone();
                let f = *generate_fn;
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (f)(&def)));
                match result {
                    Ok(content) => OutputMap::new().str("content", content).ok(),
                    Err(payload) => {
                        let message = if let Some(s) = payload.downcast_ref::<String>() {
                            s.clone()
                        } else if let Some(s) = payload.downcast_ref::<&str>() {
                            s.to_string()
                        } else {
                            "unknown panic".to_string()
                        };
                        Err(ExecError::new(format!(
                            "generate '{}' failed:\n{}",
                            name, message
                        )))
                    }
                }
            }
        }
    }
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for TestgenOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            TestgenOp::Generate { name, .. } => OutputMap::new()
                .str("content", format!("// Mock generated tests for {}", name))
                .build(),
        }
    }
}
