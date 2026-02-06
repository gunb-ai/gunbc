//! Graph builder for the testgen DAG.
//!
//! Builds a dynamic DAG with N parallel upsert chains, one per testgen target.
//! Target count is known after inventory discovery but before graph construction.

use crate::testgen_dag::ops::TestgenOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{build::*, BuilderError, Dag, DagBuilder, Node, Value};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{PrepareFileReadOp, PrepareFileWriteOp};
use gunbc_testgen_registry::TestgenTarget;
use std::collections::HashMap;
use std::path::Path;

/// The operation type for testgen graphs - a union of testgen ops, primitives, and transport.
#[derive(Debug, Clone)]
pub enum TestgenGraphOp {
    /// Testgen-specific operations
    Testgen(TestgenOp),
    /// Prepare file read (primitive - PURE)
    PrepareFileRead(PrepareFileReadOp),
    /// Prepare file write (primitive - PURE)
    PrepareFileWrite(PrepareFileWriteOp),
    /// Blob operations (compare content - PURE)
    Blob(BlobOps),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl Executable for TestgenGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            TestgenGraphOp::Testgen(op) => op.execute(inputs),
            TestgenGraphOp::PrepareFileRead(op) => op.execute(inputs),
            TestgenGraphOp::PrepareFileWrite(op) => op.execute(inputs),
            TestgenGraphOp::Blob(op) => op.execute(inputs),
            TestgenGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Build the testgen graph from discovered targets.
///
/// For each target, builds a 6-node upsert chain:
/// ```text
/// generate_{name} → prepare_read_{name} → execute_read_{name} → compare_{name}_content → execute_{name}_transport
///                 └→ prepare_write_{name} ────────────────────────────────────────────→ (request)
/// ```
///
/// All chains are independent (parallel roots).
pub fn build_testgen_graph(
    targets: &[&TestgenTarget],
    output_dir: &Path,
) -> Result<Dag<TestgenGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    for target in targets {
        let config = target.to_def();
        let name = config.name.clone();

        add_upsert_chain(
            &mut builder,
            &name,
            TestgenGraphOp::Testgen(TestgenOp::Generate {
                name: name.clone(),
                target_def: config,
                generate_fn: target.generate,
            }),
        )?;
    }

    let _ = output_dir; // paths are wired as entrypoints by the binary

    Ok(builder.build())
}

/// Build a testgen graph for testing with hardcoded mock targets.
pub fn build_testgen_graph_for_test() -> Result<Dag<TestgenGraphOp>, BuilderError> {
    use gunbc_codegen::TestgenTargetDef;

    fn mock_generate(def: &TestgenTargetDef) -> String {
        format!(
            "// Generated tests for {}\n#[cfg(test)]\nmod {} {{}}\n",
            def.name, def.module_name
        )
    }

    let targets = [
        ("mock-alpha", "mock_alpha/generated_tests.rs", "mock_alpha_generated_tests"),
        ("mock-beta", "mock_beta/generated_tests.rs", "mock_beta_generated_tests"),
    ];

    let mut builder = DagBuilder::new();

    for (name, output_path, module_name) in &targets {
        let def = TestgenTargetDef::new(name, output_path, module_name);

        add_upsert_chain(
            &mut builder,
            name,
            TestgenGraphOp::Testgen(TestgenOp::Generate {
                name: name.to_string(),
                target_def: def,
                generate_fn: mock_generate,
            }),
        )?;
    }

    Ok(builder.build())
}

/// Add a single 6-node upsert chain for a named target.
fn add_upsert_chain(
    builder: &mut DagBuilder<TestgenGraphOp>,
    name: &str,
    generate_op: TestgenGraphOp,
) -> Result<(), BuilderError> {
    let gen_id = format!("generate_{}", name);
    let prep_read_id = format!("prepare_read_{}", name);
    let exec_read_id = format!("execute_read_{}", name);
    let compare_id = format!("compare_{}_content", name);
    let prep_write_id = format!("prepare_write_{}", name);
    let exec_write_id = format!("execute_{}_transport", name);
    let response_port = format!("{}_response", name);
    let path_port = format!("{}_written_path", name);
    let content_port = format!("{}_content", name);

    // Generate node (root)
    let generate = builder.add_root_node(Node::opaque(
        gen_id.as_str(),
        vec![],
        vec![port("content", "String")],
        generate_op,
    ))?;

    // Read chain
    let prepare_read = builder.add_node_after(
        Node::opaque(
            prep_read_id.as_str(),
            vec![port("path", "String")],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            TestgenGraphOp::PrepareFileRead(PrepareFileReadOp),
        ),
        &generate,
    )?;

    let execute_read = builder.add_node_after(
        Node::opaque(
            exec_read_id.as_str(),
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![port("response", "TransportResponse")],
            TestgenGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_read,
    )?;

    // Compare
    let compare = builder.add_node_after(
        Node::opaque(
            compare_id.as_str(),
            vec![
                port("response", "TransportResponse"),
                port("expected_content", "String"),
                optional("check_mode", "Bool"),
            ],
            vec![
                port("fresh", "Bool"),
                port("skip", "Bool"),
                port("skip_reason", "String"),
            ],
            TestgenGraphOp::Blob(BlobOps::CompareContent),
        ),
        &execute_read,
    )?;

    // Write chain
    let prepare_write = builder.add_node_after(
        Node::opaque(
            prep_write_id.as_str(),
            vec![port("path", "String"), port("content", "String")],
            vec![port("request", "TransportRequest")],
            TestgenGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &generate,
    )?;

    let execute_write = builder.add_node_after(
        Node::opaque(
            exec_write_id.as_str(),
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                optional(response_port.as_str(), "TransportResponse"),
                optional(path_port.as_str(), "String"),
                optional(content_port.as_str(), "String"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            TestgenGraphOp::Transport(TransportOps::Execute),
        ),
        &compare,
    )?;

    // Wire edges
    builder.add_edge(generate.out("content"), compare.in_port("expected_content"))?;
    builder.add_edge(generate.out("content"), prepare_write.in_port("content"))?;
    builder.add_edge(prepare_read.out("request"), execute_read.in_port("request"))?;
    builder.add_edge(prepare_read.out("skip"), execute_read.in_port("skip"))?;
    builder.add_edge(execute_read.out("response"), compare.in_port("response"))?;
    builder.add_edge(compare.out("skip"), execute_write.in_port("skip"))?;
    builder.add_edge(compare.out("skip_reason"), execute_write.in_port("skip_reason"))?;
    builder.add_edge(prepare_write.out("request"), execute_write.in_port("request"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_builds_for_test() {
        let dag = build_testgen_graph_for_test().expect("graph should build");
        // 2 targets x 6 nodes = 12 nodes
        assert_eq!(dag.nodes.len(), 12);
        // 2 targets x 7 edges = 14 edges
        assert_eq!(dag.edges.len(), 14);
    }

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_testgen_graph_for_test().expect("graph should build");

        assert!(dag.get_node(&"execute_read_mock-alpha".into()).is_some());
        assert!(dag.get_node(&"execute_mock-alpha_transport".into()).is_some());
        assert!(dag.get_node(&"execute_read_mock-beta".into()).is_some());
        assert!(dag.get_node(&"execute_mock-beta_transport".into()).is_some());
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_testgen_graph_for_test().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        assert!(entrypoints.is_entrypoint_port(&"compare_mock-alpha_content".into(), &"check_mode".into()));
        assert!(entrypoints.is_entrypoint_port(&"compare_mock-beta_content".into(), &"check_mode".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_read_mock-alpha".into(), &"path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_read_mock-beta".into(), &"path".into()));
    }

    #[test]
    fn test_pure_nodes_not_boundaries() {
        let dag = build_testgen_graph_for_test().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        assert!(!boundaries.is_boundary_node(&"generate_mock-alpha".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_mock-alpha".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_write_mock-alpha".into()));
    }
}
