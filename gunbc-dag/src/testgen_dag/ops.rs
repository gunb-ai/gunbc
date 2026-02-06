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
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            TestgenOp::Generate {
                target_def,
                generate_fn,
                ..
            } => {
                let content = (generate_fn)(target_def);
                OutputMap::new().str("content", content).ok()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_generate(_def: &TestgenTargetDef) -> String {
        "// mock test content".to_string()
    }

    #[test]
    fn test_generate_op() {
        let def = TestgenTargetDef::new("test", "test/output.rs", "test_module");
        let op = TestgenOp::Generate {
            name: "test".to_string(),
            target_def: def,
            generate_fn: mock_generate,
        };
        let result = op.execute(HashMap::new()).unwrap();
        match result.get("content") {
            Some(Value::Str(content)) => {
                assert!(content.contains("mock test content"));
            }
            _ => panic!("expected content string"),
        }
    }
}
