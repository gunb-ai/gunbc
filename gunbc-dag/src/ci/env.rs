//! Environment operation for runner tool acquisition.
//!
//! The `EnvOp` represents the runner environment and is responsible for
//! acquiring tools before they're used by downstream nodes. This is the
//! I/O boundary for tool acquisition - it performs check/install and
//! emits ToolHandles that flow through the DAG via edges.
//!
//! # Design
//!
//! Resources flow from owners to consumers:
//! ```text
//! runner_env (EnvOp)
//!     │
//!     │ tool:clippy → ToolHandle
//!     ▼
//! lint_node (receives handle via edge)
//! ```
//!
//! The env node is the I/O boundary - it gets mocked in DryRun mode.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::cli::{upsert_tool, get_tool_by_id, ToolHandle, WhichResolver};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Environment operation that acquires tools for downstream nodes.
///
/// This is the I/O boundary for tool acquisition. On execute:
/// 1. For each tool ID, run the upsert pattern (check/install)
/// 2. Emit ToolHandles with resolved paths
///
/// In DryRun mode, this node should be intercepted with mock ToolHandles.
#[derive(Debug, Clone)]
pub struct EnvOp {
    /// Tool IDs that this environment provides
    pub tools: Vec<&'static str>,
}

impl EnvOp {
    /// Create a new environment op that provides the given tools.
    pub fn new(tools: Vec<&'static str>) -> Self {
        Self { tools }
    }

    /// Create an environment for CI (provides common CI tools).
    pub fn ci() -> Self {
        Self::new(vec!["cargo", "clippy"])
    }

    /// Get the output port names for this environment.
    pub fn output_ports(&self) -> Vec<String> {
        self.tools.iter().map(|t| format!("tool:{}", t)).collect()
    }
}

impl Executable for EnvOp {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let mut outputs = HashMap::new();

        for tool_id in &self.tools {
            // Look up the tool definition
            let tool = get_tool_by_id(tool_id).ok_or_else(|| {
                ExecError::new(format!("Unknown tool '{}' in environment", tool_id))
            })?;

            // DI violation: WhichResolver constructed inline.
            // Phase 2 will acquire path resolution through DAG input ports.
            let resolver = WhichResolver;
            let path = upsert_tool(tool, &resolver)
                .map_err(|e| ExecError::new(format!("Failed to acquire tool '{}': {}", tool_id, e)))?;

            // Create handle and add to outputs
            let handle = ToolHandle::acquire(tool, path);
            let port_name = format!("tool:{}", tool_id);
            outputs.insert(port_name, handle.into());
        }

        Ok(outputs)
    }
}

/// Create mock outputs for EnvOp in DryRun mode.
///
/// Returns mock ToolHandles for each tool the environment provides.
pub fn mock_env_outputs(env: &EnvOp) -> HashMap<String, Value> {
    let mut outputs = HashMap::new();

    for tool_id in &env.tools {
        if let Some(tool) = get_tool_by_id(tool_id) {
            let handle = ToolHandle::mock(tool);
            let port_name = format!("tool:{}", tool_id);
            outputs.insert(port_name, handle.into());
        }
    }

    outputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_op_output_ports() {
        let env = EnvOp::new(vec!["cargo", "clippy"]);
        let ports = env.output_ports();
        assert_eq!(ports, vec!["tool:cargo", "tool:clippy"]);
    }

    #[test]
    fn test_env_ci_preset() {
        let env = EnvOp::ci();
        assert!(env.tools.contains(&"cargo"));
        assert!(env.tools.contains(&"clippy"));
    }

    #[test]
    fn test_mock_env_outputs() {
        let env = EnvOp::new(vec!["cargo", "clippy"]);
        let outputs = mock_env_outputs(&env);

        assert!(outputs.contains_key("tool:cargo"));
        assert!(outputs.contains_key("tool:clippy"));

        // Check that mock paths are used
        if let Value::Str(s) = outputs.get("tool:cargo").unwrap() {
            assert!(s.contains("/mock/"));
        }
    }
}
