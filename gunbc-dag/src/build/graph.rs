//! Graph builder for the build pipeline.
//!
//! Pipeline:
//! ```text
//! PrepareBuild → ExecuteBuild → ParseBuild
//!                                   ↓
//!                     ┌─────────────┴─────────────┐
//!                     ↓                           ↓
//! PrepareTest → ExecuteTest      PrepareClippy → ExecuteClippy
//!                  ↓                               ↓
//!              ParseTest                       ParseClippy
//!                     ↓                           ↓
//!                     └─────────────┬─────────────┘
//!                                   ↓
//!                               Summary
//! ```

use crate::build::ops::BuildOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    add_skippable_transport_triplet, add_transport_triplet,
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_transport::TransportOps;
use std::collections::HashMap;

/// Union type for build graph operations.
#[derive(Debug, Clone)]
pub enum BuildGraphOp {
    /// Build-specific pure operations.
    Build(BuildOp),
    /// Transport operations (boundary - actual I/O).
    Transport(TransportOps),
}

impl Executable for BuildGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            BuildGraphOp::Build(op) => op.execute(inputs),
            BuildGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Get the declared signature for the build workflow.
pub fn build_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        .with_output("overall_success", "Bool", Cardinality::ONE)
        .with_output("report", "String", Cardinality::ONE)
}

/// Build the build graph: build → (test + clippy) → summary.
pub fn build_build_graph() -> Result<Dag<BuildGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // Build Stage
    // ========================================================================

    let build = add_transport_triplet(
        &mut builder,
        "build",
        vec![],
        vec![
            port("build_success", "Bool"),
            port("build_stdout", "String"),
            port("build_stderr", "String"),
        ],
        BuildGraphOp::Build(BuildOp::PrepareBuild),
        BuildGraphOp::Build(BuildOp::ParseBuild),
        BuildGraphOp::Transport(TransportOps::Execute),
        None,
    )?;

    // ========================================================================
    // Test Stage (parallel with Clippy, both depend on build)
    // ========================================================================

    let test = add_skippable_transport_triplet(
        &mut builder,
        "test",
        vec![port("build_success", "Bool")],
        vec![
            port("test_success", "Bool"),
            port("test_skipped", "Bool"),
            port("test_stdout", "String"),
            port("test_stderr", "String"),
        ],
        BuildGraphOp::Build(BuildOp::PrepareTest),
        BuildGraphOp::Build(BuildOp::ParseTest),
        BuildGraphOp::Transport(TransportOps::Execute),
        &build.parse,
    )?;

    // ========================================================================
    // Clippy Stage (parallel with Test)
    // ========================================================================

    let clippy = add_skippable_transport_triplet(
        &mut builder,
        "clippy",
        vec![port("build_success", "Bool")],
        vec![
            port("clippy_success", "Bool"),
            port("clippy_skipped", "Bool"),
            port("clippy_stdout", "String"),
            port("clippy_stderr", "String"),
        ],
        BuildGraphOp::Build(BuildOp::PrepareClippy),
        BuildGraphOp::Build(BuildOp::ParseClippy),
        BuildGraphOp::Transport(TransportOps::Execute),
        &build.parse,
    )?;

    // ========================================================================
    // Summary Stage (depends on both test and clippy)
    // ========================================================================

    let summary = builder.add_node_after_all(
        Node::opaque(
            "summary",
            vec![
                port("build_success", "Bool"),
                port("test_success", "Bool"),
                port("clippy_success", "Bool"),
                optional("build_stderr", "String"),
                optional("test_stderr", "String"),
                optional("clippy_stderr", "String"),
            ],
            vec![port("overall_success", "Bool"), port("report", "String")],
            BuildGraphOp::Build(BuildOp::Summary),
        ),
        &[&test.parse, &clippy.parse],
    )?;

    // ========================================================================
    // Wire up cross-triplet edges (internal edges handled by helpers)
    // ========================================================================

    // Test stage — build feeds prepare
    builder.add_edge(
        build.parse.out("build_success"),
        test.prepare.in_port("build_success"),
    )?;

    // Clippy stage — build feeds prepare
    builder.add_edge(
        build.parse.out("build_success"),
        clippy.prepare.in_port("build_success"),
    )?;

    // Summary stage
    builder.add_edge(
        build.parse.out("build_success"),
        summary.in_port("build_success"),
    )?;
    builder.add_edge(
        test.parse.out("test_success"),
        summary.in_port("test_success"),
    )?;
    builder.add_edge(
        clippy.parse.out("clippy_success"),
        summary.in_port("clippy_success"),
    )?;
    builder.add_edge(
        build.parse.out("build_stderr"),
        summary.in_port("build_stderr"),
    )?;
    builder.add_edge(
        test.parse.out("test_stderr"),
        summary.in_port("test_stderr"),
    )?;
    builder.add_edge(
        clippy.parse.out("clippy_stderr"),
        summary.in_port("clippy_stderr"),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::detect_boundaries;

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_build_graph().expect("graph should build");
        // 10 nodes: prepare/execute/parse × build + test + clippy + summary
        assert_eq!(dag.nodes.len(), 10);
    }

    #[test]
    fn test_graph_has_transport_nodes() {
        let dag = build_build_graph().expect("graph should build");
        let transport_nodes: Vec<_> = dag
            .nodes
            .iter()
            .filter(|n| n.id.0.starts_with("execute_"))
            .collect();
        // 3 transport nodes: execute_build, execute_test, execute_clippy
        assert_eq!(transport_nodes.len(), 3);
    }

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_build_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);
        // Summary should be a boundary (its outputs leave the DAG)
        assert!(boundaries.is_boundary_node(&"summary".into()));
    }

    #[test]
    fn test_graph_has_parallel_stages() {
        let dag = build_build_graph().expect("graph should build");
        // prepare_test and prepare_clippy should both depend on parse_build
        let test_parents: Vec<_> = dag
            .edges
            .iter()
            .filter(|e| e.to_node == "prepare_test".into())
            .map(|e| &e.from_node)
            .collect();
        let clippy_parents: Vec<_> = dag
            .edges
            .iter()
            .filter(|e| e.to_node == "prepare_clippy".into())
            .map(|e| &e.from_node)
            .collect();
        assert!(test_parents.iter().any(|n| n.0 == "parse_build"));
        assert!(clippy_parents.iter().any(|n| n.0 == "parse_build"));
    }
}
