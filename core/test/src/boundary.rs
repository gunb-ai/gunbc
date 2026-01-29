//! Boundary test helpers.
//!
//! Boundary tests verify that a DAG's world-write boundaries can be mocked.

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_ir::{detect_boundaries, Dag, Value};

/// Result of a boundary test.
#[derive(Debug)]
pub struct BoundaryTestResult {
    /// Whether the test passed
    pub success: bool,
    /// Boundary nodes that were tested
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

/// Assert that a DAG's boundaries can be mocked in dry-run mode.
///
/// This test verifies that:
/// 1. The DAG has identifiable boundaries
/// 2. Executing in dry-run mode succeeds
/// 3. All boundary nodes were intercepted
///
/// # Example
///
/// ```ignore
/// let dag = build_gist_graph();
/// let result = assert_boundary_mockable(&dag, default_mocks());
/// assert!(result.is_ok());
/// ```
pub fn assert_boundary_mockable<T: gunbc_exec::Executable + Clone>(
    dag: &Dag<T>,
    mocks: BoundaryMocks,
) -> BoundaryTestResult {
    // Detect boundaries
    let boundaries = detect_boundaries(dag);

    if boundaries.boundary_nodes.is_empty() {
        return BoundaryTestResult {
            success: false,
            boundary_nodes: vec![],
            error: Some("DAG has no boundaries — nothing to test".to_string()),
        };
    }

    let boundary_nodes: Vec<String> = boundaries.boundary_nodes.iter().map(|n| n.0.clone()).collect();

    // Execute in dry-run mode
    match execute_with_mode(dag, ExecutionMode::DryRun(mocks)) {
        Ok(log) => {
            // Verify all boundaries were intercepted
            let mut all_intercepted = true;
            for node_id in &boundary_nodes {
                if let Some(entry) = log.get(node_id) {
                    if !entry.was_intercepted {
                        all_intercepted = false;
                    }
                }
            }

            if all_intercepted {
                BoundaryTestResult {
                    success: true,
                    boundary_nodes,
                    error: None,
                }
            } else {
                BoundaryTestResult {
                    success: false,
                    boundary_nodes,
                    error: Some("Not all boundary nodes were intercepted".to_string()),
                }
            }
        }
        Err(e) => BoundaryTestResult {
            success: false,
            boundary_nodes,
            error: Some(format!("Execution failed: {}", e)),
        },
    }
}

/// Create default mocks for boundary testing.
///
/// Returns mocks that produce a default "<DRY-RUN>" string for all boundary ports.
pub fn default_mocks() -> BoundaryMocks {
    BoundaryMocks::with_default(Value::Str("<DRY-RUN>".to_string()))
}

/// Create mocks with specific values for known boundary ports.
pub fn mocks_with_values(values: impl IntoIterator<Item = (&'static str, &'static str, Value)>) -> BoundaryMocks {
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
    use gunbc_ir::Node;

    #[test]
    fn test_boundary_mockable_passes() {
        let mut dag: Dag<MockOp> = Dag::new();
        dag.add_node(Node::opaque(
            "sink",
            vec![],
            vec![port("out", "S")],
            MockOp::new("sink", [("out", Value::Str("real".to_string()))]),
        ));

        let result = assert_boundary_mockable(&dag, default_mocks());
        assert!(result.is_ok());
        assert_eq!(result.boundary_nodes, vec!["sink"]);
    }

    #[test]
    fn test_no_boundaries_fails() {
        // Empty DAG has no boundaries
        let dag: Dag<MockOp> = Dag::new();
        let result = assert_boundary_mockable(&dag, default_mocks());
        assert!(!result.is_ok());
        assert!(result.error.unwrap().contains("no boundaries"));
    }
}
