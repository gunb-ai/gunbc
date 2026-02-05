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
#[allow(clippy::result_large_err)]
pub fn build_build_graph() -> Result<Dag<BuildGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // Build Stage
    // ========================================================================

    let prepare_build = builder.add_root_node(Node::opaque(
        "prepare_build",
        vec![],
        vec![port("request", "TransportRequest")],
        BuildGraphOp::Build(BuildOp::PrepareBuild),
    ))?;

    let execute_build = builder.add_node_after(
        Node::opaque(
            "execute_build",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            BuildGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_build,
    )?;

    let parse_build = builder.add_node_after(
        Node::opaque(
            "parse_build",
            vec![port("response", "TransportResponse")],
            vec![
                port("build_success", "Bool"),
                port("build_stdout", "String"),
                port("build_stderr", "String"),
            ],
            BuildGraphOp::Build(BuildOp::ParseBuild),
        ),
        &execute_build,
    )?;

    // ========================================================================
    // Test Stage (parallel with Clippy, both depend on build)
    // ========================================================================

    let prepare_test = builder.add_node_after(
        Node::opaque(
            "prepare_test",
            vec![port("build_success", "Bool")],
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            BuildGraphOp::Build(BuildOp::PrepareTest),
        ),
        &parse_build,
    )?;

    let execute_test = builder.add_node_after(
        Node::opaque(
            "execute_test",
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
            ],
            BuildGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_test,
    )?;

    let parse_test = builder.add_node_after(
        Node::opaque(
            "parse_test",
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
            ],
            vec![
                port("test_success", "Bool"),
                port("test_skipped", "Bool"),
                port("test_stdout", "String"),
                port("test_stderr", "String"),
            ],
            BuildGraphOp::Build(BuildOp::ParseTest),
        ),
        &execute_test,
    )?;

    // ========================================================================
    // Clippy Stage (parallel with Test)
    // ========================================================================

    let prepare_clippy = builder.add_node_after(
        Node::opaque(
            "prepare_clippy",
            vec![port("build_success", "Bool")],
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            BuildGraphOp::Build(BuildOp::PrepareClippy),
        ),
        &parse_build,
    )?;

    let execute_clippy = builder.add_node_after(
        Node::opaque(
            "execute_clippy",
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
            ],
            BuildGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_clippy,
    )?;

    let parse_clippy = builder.add_node_after(
        Node::opaque(
            "parse_clippy",
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
            ],
            vec![
                port("clippy_success", "Bool"),
                port("clippy_skipped", "Bool"),
                port("clippy_stdout", "String"),
                port("clippy_stderr", "String"),
            ],
            BuildGraphOp::Build(BuildOp::ParseClippy),
        ),
        &execute_clippy,
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
        &[&parse_test, &parse_clippy],
    )?;

    // ========================================================================
    // Wire up edges
    // ========================================================================

    // Build stage
    builder.add_edge(
        prepare_build.out("request"),
        execute_build.in_port("request"),
    )?;
    builder.add_edge(
        execute_build.out("response"),
        parse_build.in_port("response"),
    )?;

    // Test stage
    builder.add_edge(
        parse_build.out("build_success"),
        prepare_test.in_port("build_success"),
    )?;
    builder.add_edge(prepare_test.out("request"), execute_test.in_port("request"))?;
    builder.add_edge(prepare_test.out("skip"), execute_test.in_port("skip"))?;
    builder.add_edge(execute_test.out("response"), parse_test.in_port("response"))?;
    builder.add_edge(execute_test.out("skip"), parse_test.in_port("skip"))?;

    // Clippy stage
    builder.add_edge(
        parse_build.out("build_success"),
        prepare_clippy.in_port("build_success"),
    )?;
    builder.add_edge(
        prepare_clippy.out("request"),
        execute_clippy.in_port("request"),
    )?;
    builder.add_edge(prepare_clippy.out("skip"), execute_clippy.in_port("skip"))?;
    builder.add_edge(
        execute_clippy.out("response"),
        parse_clippy.in_port("response"),
    )?;
    builder.add_edge(execute_clippy.out("skip"), parse_clippy.in_port("skip"))?;

    // Summary stage
    builder.add_edge(
        parse_build.out("build_success"),
        summary.in_port("build_success"),
    )?;
    builder.add_edge(
        parse_test.out("test_success"),
        summary.in_port("test_success"),
    )?;
    builder.add_edge(
        parse_clippy.out("clippy_success"),
        summary.in_port("clippy_success"),
    )?;
    builder.add_edge(
        parse_build.out("build_stderr"),
        summary.in_port("build_stderr"),
    )?;
    builder.add_edge(
        parse_test.out("test_stderr"),
        summary.in_port("test_stderr"),
    )?;
    builder.add_edge(
        parse_clippy.out("clippy_stderr"),
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
