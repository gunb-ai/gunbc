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
use gunbc_ir::transport::cli::{
    get_tool_by_id, upsert_tool_with, ToolHandle, ToolPathResolver, WhichResolver,
};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Environment operation that acquires tools for downstream nodes.
///
/// This is the I/O boundary for tool acquisition. On execute:
/// 1. For each tool ID, run the upsert pattern (check/install)
/// 2. Emit ToolHandles with resolved paths
/// 3. Optionally emit an `exec_mode` string for downstream nodes
///
/// In DryRun mode, this node should be intercepted with mock ToolHandles.
#[derive(Debug, Clone)]
pub struct EnvOp {
    /// Tool IDs that this environment provides
    pub tools: Vec<&'static str>,
    /// Optional exec mode string to emit as an output (e.g., "ensure", "verify").
    /// When set, the env node emits this as the `exec_mode` output port.
    pub exec_mode: Option<String>,
}

impl EnvOp {
    /// Create a new environment op that provides the given tools.
    pub fn new(tools: Vec<&'static str>) -> Self {
        Self {
            tools,
            exec_mode: None,
        }
    }

    /// Create an environment op with an exec mode to emit.
    pub fn with_exec_mode(tools: Vec<&'static str>, mode_str: &str) -> Self {
        Self {
            tools,
            exec_mode: Some(mode_str.to_string()),
        }
    }

    /// Create an environment for CI (provides common CI tools).
    pub fn ci() -> Self {
        Self::new(vec!["cargo", "clippy"])
    }

    /// Get the output port names for this environment.
    pub fn output_ports(&self) -> Vec<String> {
        let mut ports: Vec<String> = self.tools.iter().map(|t| format!("tool:{}", t)).collect();
        if self.exec_mode.is_some() {
            ports.push("exec_mode".to_string());
        }
        ports
    }

    /// Execute tool acquisition with a specific path resolver.
    ///
    /// This is the injectable variant for testing. Pass a [`MockResolver`]
    /// to avoid shelling out to `which`.
    ///
    /// [`MockResolver`]: gunbc_ir::transport::cli::MockResolver
    pub fn execute_with_resolver(
        &self,
        resolver: &dyn ToolPathResolver,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let mut outputs = HashMap::new();

        for tool_id in &self.tools {
            let tool = get_tool_by_id(tool_id).ok_or_else(|| {
                ExecError::new(format!("Unknown tool '{}' in environment", tool_id))
            })?;

            let path = upsert_tool_with(tool, resolver).map_err(|e| {
                ExecError::new(format!("Failed to acquire tool '{}': {}", tool_id, e))
            })?;

            let handle = ToolHandle::acquire(tool, path);
            let port_name = format!("tool:{}", tool_id);
            outputs.insert(port_name, handle.into());
        }

        if let Some(ref mode) = self.exec_mode {
            outputs.insert("exec_mode".to_string(), Value::Str(mode.clone()));
        }

        Ok(outputs)
    }
}

impl Executable for EnvOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        self.execute_with_resolver(&WhichResolver)
    }
}

/// Create mock outputs for EnvOp in DryRun mode.
///
/// Returns mock ToolHandles for each tool the environment provides,
/// plus the `exec_mode` output if configured.
pub fn mock_env_outputs(env: &EnvOp) -> HashMap<String, Value> {
    let mut outputs = HashMap::new();

    for tool_id in &env.tools {
        if let Some(tool) = get_tool_by_id(tool_id) {
            let handle = ToolHandle::mock(tool);
            let port_name = format!("tool:{}", tool_id);
            outputs.insert(port_name, handle.into());
        }
    }

    if let Some(ref mode) = env.exec_mode {
        outputs.insert("exec_mode".to_string(), Value::Str(mode.clone()));
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
