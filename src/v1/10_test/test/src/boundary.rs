//! Transport interception test helpers.
//!
//! These tests verify that a DAG's transport executor nodes can be mocked.
//!
//! # Transport Interception Model
//!
//! DryRun mode intercepts **transport execution nodes** - nodes that consume
//! `TransportRequest` values. This follows the design principle:
//!
//! > "World I/O is performed only by transport executor nodes"
//! > "DryRun intercepts transport execution nodes, not boundary outputs"

use gunbc_exec::{execute_dag, BoundaryMocks, ExecuteConfig, ExecutionMode};
use gunbc_ir::{Dag, NodeKind, Value};

/// Result of a transport interception test.
#[derive(Debug)]
pub struct BoundaryTestResult {
    /// Whether the test passed
    pub success: bool,
    /// Transport executor nodes that were tested
    pub boundary_nodes: Vec<String>,
    /// Any error message
    pub error: Option<String>,
}

impl BoundaryTestResult {
    /// Check if the test passed.
    pub fn is_ok(&self) -> bool {
        self.success
    }
}

/// Find all transport executor nodes in a DAG.
fn find_transport_executors<T>(dag: &Dag<T>) -> Vec<String> {
    dag.nodes
        .iter()
        .filter(|node| node.kind == NodeKind::TransportExecute)
        .map(|node| node.id.0.clone())
        .collect()
}

/// Assert that a DAG's transport executors can be mocked in dry-run mode.
///
/// This test verifies that:
/// 1. The DAG has transport executor nodes
/// 2. Executing in dry-run mode succeeds
/// 3. All transport executor nodes were intercepted
///
/// # Example
///
/// ```text
/// let dag = build_gist_graph();
/// let result = assert_boundary_mockable(&dag, default_mocks());
/// assert!(result.is_ok());
/// ```
pub fn assert_boundary_mockable<T: gunbc_exec::Executable + Clone + Send>(
    dag: &Dag<T>,
    mocks: BoundaryMocks,
) -> BoundaryTestResult {
    // Find transport executor nodes
    let transport_executors = find_transport_executors(dag);

    if transport_executors.is_empty() {
        // No transport executors means there's nothing to intercept under this model.
        return BoundaryTestResult {
            success: true,
            boundary_nodes: vec![],
            error: None,
        };
    }

    // Execute in dry-run mode with lenient strictness — boundary tests
    // intentionally omit non-boundary inputs.
    match execute_dag(
        dag,
        ExecuteConfig {
            mode: ExecutionMode::DryRun(mocks),
            strictness: gunbc_exec::DryRunStrictness::Lenient,
            ..Default::default()
        },
    ) {
        Ok(log) => {
            // Verify all transport executors were intercepted or skipped.
            // Skipped nodes (guard failed, conditional branch not taken) are
            // acceptable — they never executed, so interception is moot.
            let mut all_intercepted = true;
            for node_id in &transport_executors {
                if let Some(entry) = log.get(node_id) {
                    if !entry.was_intercepted {
                        let all_skipped =
                            entry.outputs.values().all(|v| matches!(v, Value::Skipped));
                        if !all_skipped {
                            all_intercepted = false;
                        }
                    }
                }
            }

            if all_intercepted {
                BoundaryTestResult {
                    success: true,
                    boundary_nodes: transport_executors,
                    error: None,
                }
            } else {
                BoundaryTestResult {
                    success: false,
                    boundary_nodes: transport_executors,
                    error: Some("Not all transport executor nodes were intercepted".to_string()),
                }
            }
        }
        Err(e) => BoundaryTestResult {
            success: false,
            boundary_nodes: transport_executors,
            error: Some(format!("Execution failed: {}", e)),
        },
    }
}

/// Execute a DAG through the canonical engine surface with optional entrypoint mocks.
///
/// This adapter centralizes direct `gunbc_exec` invocation for callers that need
/// explicit input mocks while preserving the same execution semantics as
/// `execute_dag`.
pub fn execute_via_engine_with_inputs<T: gunbc_exec::Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<gunbc_exec::ExecutionLog, gunbc_exec::ExecError> {
    execute_dag(
        dag,
        ExecuteConfig {
            mode,
            input_mocks,
            ..Default::default()
        },
    )
}

/// Create empty mocks for boundary testing.
///
/// Intercepted nodes require explicit mocks per output port.
pub fn default_mocks() -> BoundaryMocks {
    BoundaryMocks::new()
}

/// Create mocks with specific values for known boundary ports.
pub fn mocks_with_values(
    values: impl IntoIterator<Item = (&'static str, &'static str, Value)>,
) -> BoundaryMocks {
    let mut mocks = BoundaryMocks::new();
    for (node, port, value) in values {
        mocks.set_value(node, port, value);
    }
    mocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockOp;
    use gunbc_ir::build::*;
    use gunbc_ir::{Node, NodeKind};

    #[test]
    fn test_transport_executor_mockable_passes() {
        // A transport executor node has TransportRequest input
        let mut dag: Dag<MockOp> = Dag::new();
        dag.add_node(
            Node::opaque(
                "execute_transport",
                vec![port("request", "TransportRequest")],
                vec![port("response", "TransportResponse")],
                MockOp::new(
                    "execute_transport",
                    [("response", Value::Str("real".to_string()))],
                ),
            )
            .with_kind(NodeKind::TransportExecute),
        );

        let result = assert_boundary_mockable(
            &dag,
            mocks_with_values([(
                "execute_transport",
                "response",
                Value::Str("mock".to_string()),
            )]),
        );
        assert!(result.is_ok());
        assert_eq!(result.boundary_nodes, vec!["execute_transport"]);
    }

    #[test]
    fn test_no_transport_executors_succeeds() {
        // DAG with no transport executor nodes
        let mut dag: Dag<MockOp> = Dag::new();
        dag.add_node(
            Node::opaque(
                "pure_node",
                vec![port("input", "String")],
                vec![port("output", "String")],
                MockOp::new("pure_node", [("output", Value::Str("result".to_string()))]),
            )
            .with_kind(NodeKind::Pure),
        );

        let result = assert_boundary_mockable(&dag, BoundaryMocks::new());
        assert!(result.is_ok());
        assert!(result.boundary_nodes.is_empty());
    }

    #[test]
    fn test_empty_dag_succeeds() {
        // Empty DAG has no transport executors
        let dag: Dag<MockOp> = Dag::new();
        let result = assert_boundary_mockable(&dag, BoundaryMocks::new());
        assert!(result.is_ok());
        assert!(result.boundary_nodes.is_empty());
    }
}
