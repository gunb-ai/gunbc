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
use crate::{add_fs_env_root_node, wire_fs_env_write_edges};
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    add_skippable_transport_triplet, add_transport_triplet, build::*, BuilderError, Cardinality,
    Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::FsEnv;
use std::collections::HashMap;

/// Union type for build graph operations.
#[derive(Debug, Clone)]
pub enum BuildGraphOp {
    /// Build-specific pure operations.
    Build(BuildOp),
    /// Filesystem environment (resource acquisition).
    FsEnv(FsEnv),
    /// Transport operations (boundary - actual I/O).
    Transport(TransportOps),
}

impl Executable for BuildGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            BuildGraphOp::Build(op) => op.execute(inputs),
            BuildGraphOp::FsEnv(op) => op.execute(inputs),
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
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "build",
    builder = "build_build_graph().unwrap()"
)]
pub fn build_build_graph() -> Result<Dag<BuildGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // Environment: filesystem handle
    // ========================================================================

    let fs_env = add_fs_env_root_node(&mut builder, BuildGraphOp::FsEnv)?;

    let fs_resource = resource("file", "FilesystemHandle", AccessMode::Write);

    // ========================================================================
    // Build Stage
    // ========================================================================

    let build = add_transport_triplet(
        &mut builder,
        "build",
        vec![],
        vec![fs_resource.clone()],
        vec![
            port("build_success", "Bool"),
            port("build_stdout", "String"),
            port("build_stderr", "String"),
        ],
        BuildGraphOp::Build(BuildOp::PrepareBuild),
        BuildGraphOp::Build(BuildOp::ParseBuild),
        BuildGraphOp::Transport(TransportOps::Execute),
        Some(&fs_env),
    )?;

    // ========================================================================
    // Test Stage (parallel with Clippy, both depend on build)
    // ========================================================================

    let test = add_skippable_transport_triplet(
        &mut builder,
        "test",
        vec![port("build_success", "Bool")],
        vec![fs_resource.clone()],
        vec![
            port("test_success", "Bool"),
            port("test_skipped", "Bool"),
            port("test_stdout", "String"),
            port("test_stderr", "String"),
        ],
        BuildGraphOp::Build(BuildOp::PrepareTest),
        BuildGraphOp::Build(BuildOp::ParseTest),
        BuildGraphOp::Transport(TransportOps::Execute),
        &build,
    )?;

    // ========================================================================
    // Clippy Stage (parallel with Test)
    // ========================================================================

    let clippy = add_skippable_transport_triplet(
        &mut builder,
        "clippy",
        vec![port("build_success", "Bool")],
        vec![fs_resource.clone()],
        vec![
            port("clippy_success", "Bool"),
            port("clippy_skipped", "Bool"),
            port("clippy_stdout", "String"),
            port("clippy_stderr", "String"),
        ],
        BuildGraphOp::Build(BuildOp::PrepareClippy),
        BuildGraphOp::Build(BuildOp::ParseClippy),
        BuildGraphOp::Transport(TransportOps::Execute),
        &build,
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
                optional("build_stderr", "OptionalString"),
                optional("test_stderr", "OptionalString"),
                optional("clippy_stderr", "OptionalString"),
            ],
            vec![port("overall_success", "Bool"), port("report", "String")],
            BuildGraphOp::Build(BuildOp::Summary),
        ),
        &[&test, &clippy],
    )?;

    // ========================================================================
    // Wire up cross-triplet edges (internal edges handled by helpers)
    // ========================================================================

    // Test stage — build feeds prepare
    builder.add_edge(build.out("build_success"), test.in_port("build_success"))?;

    // Clippy stage — build feeds prepare
    builder.add_edge(build.out("build_success"), clippy.in_port("build_success"))?;

    // Summary stage
    builder.add_edge(build.out("build_success"), summary.in_port("build_success"))?;
    builder.add_edge(test.out("test_success"), summary.in_port("test_success"))?;
    builder.add_edge(
        clippy.out("clippy_success"),
        summary.in_port("clippy_success"),
    )?;
    builder.add_edge(build.out("build_stderr"), summary.in_port("build_stderr"))?;
    builder.add_edge(test.out("test_stderr"), summary.in_port("test_stderr"))?;
    builder.add_edge(
        clippy.out("clippy_stderr"),
        summary.in_port("clippy_stderr"),
    )?;

    // Resource wiring
    wire_fs_env_write_edges(
        &mut builder,
        &fs_env,
        vec![
            build.in_port("res:file"),
            test.in_port("res:file"),
            clippy.in_port("res:file"),
        ],
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, NodeBody};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_build_graph().expect("graph should build");
        // Transport triplets are now SubDag nodes
        for subdag_name in ["build", "test", "clippy"] {
            let node = dag
                .get_node(&subdag_name.into())
                .unwrap_or_else(|| panic!("missing SubDag node: {}", subdag_name));
            assert!(node.is_subdag(), "{} should be a SubDag", subdag_name);
        }
        // Non-SubDag nodes
        assert!(
            dag.get_node(&"summary".into()).is_some(),
            "missing node: summary"
        );
    }

    #[test]
    fn test_graph_has_transport_nodes() {
        let dag = build_build_graph().expect("graph should build");
        for (subdag_name, execute_name) in [
            ("build", "execute_build"),
            ("test", "execute_test"),
            ("clippy", "execute_clippy"),
        ] {
            let subdag_node = dag
                .get_node(&subdag_name.into())
                .unwrap_or_else(|| panic!("missing SubDag: {}", subdag_name));
            if let NodeBody::SubDag(ref inner) = subdag_node.body {
                let execute_node = inner.get_node(&execute_name.into()).unwrap_or_else(|| {
                    panic!(
                        "missing transport node {} inside {}",
                        execute_name, subdag_name
                    )
                });
                assert!(
                    matches!(
                        execute_node.body,
                        NodeBody::Opaque(BuildGraphOp::Transport(_))
                    ),
                    "{} should be a transport node",
                    execute_name
                );
            } else {
                panic!("{} should be a SubDag", subdag_name);
            }
        }
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
        // test and clippy SubDags should both depend on the build SubDag
        let test_parents: Vec<_> = dag
            .edges
            .iter()
            .filter(|e| e.to_node == "test".into())
            .map(|e| &e.from_node)
            .collect();
        let clippy_parents: Vec<_> = dag
            .edges
            .iter()
            .filter(|e| e.to_node == "clippy".into())
            .map(|e| &e.from_node)
            .collect();
        assert!(test_parents.iter().any(|n| n.0 == "build"));
        assert!(clippy_parents.iter().any(|n| n.0 == "build"));
    }
}
