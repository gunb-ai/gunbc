//! Graph builder for the Buck2 tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.

use crate::ops::Buck2Op;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrepareFileWriteOp;
use std::collections::HashMap;

/// The operation type for buck2 graphs - a union of primitives, library, and tool-specific ops.
#[derive(Debug, Clone)]
pub enum Buck2GraphOp {
    /// Buck2-specific operations
    Buck2(Buck2Op),
    /// Prepare file write (primitive)
    PrepareFileWrite(PrepareFileWriteOp),
    /// Transport operations (from gunbc-ops)
    Transport(TransportOps),
}

impl Executable for Buck2GraphOp {
    fn execute(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            Buck2GraphOp::Buck2(op) => op.execute(inputs),
            Buck2GraphOp::PrepareFileWrite(op) => op.execute(inputs),
            Buck2GraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Get the declared signature for the buck2 workflow.
pub fn buck2_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("cargo_toml_path", "String", Cardinality::One)
        .with_input("output_path", "String", Cardinality::One)
        // Outputs from execute_transport (boundary)
        .with_output("response", "TransportResponse", Cardinality::One)
        .with_output("written_path", "String", Cardinality::One)
        .with_output("content", "String", Cardinality::One)
}

/// Build the Buck2 generation graph using DagBuilder.
///
/// Pipeline:
/// ```text
/// ParseCargoToml -> ExtractDeps -> GenerateBuckTargets -> PrepareFileWrite -> ExecuteTransport
///    (buck2)        (buck2)           (buck2)               (fs)              (transport)
/// ```
pub fn build_buck2_graph() -> Result<Dag<Buck2GraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: ParseCargoToml (buck2-specific) - generation 0
    let parse_cargo_toml = builder.add_root_node(Node::opaque(
        "parse_cargo_toml",
        vec![port("cargo_toml_path", "String")],
        vec![port("cargo_toml", "Json")],
        Buck2GraphOp::Buck2(Buck2Op::ParseCargoToml),
    ))?;

    // Node: ExtractDeps (buck2-specific) - generation 1
    let extract_deps = builder.add_node_after(
        Node::opaque(
            "extract_deps",
            vec![port("cargo_toml", "Json")],
            vec![port("members", "StrList"), port("deps", "MapStrStr")],
            Buck2GraphOp::Buck2(Buck2Op::ExtractDeps),
        ),
        &parse_cargo_toml,
    )?;

    // Node: GenerateBuckTargets (buck2-specific) - generation 2
    let generate_targets = builder.add_node_after(
        Node::opaque(
            "generate_targets",
            vec![port("members", "StrList"), port("deps", "MapStrStr")],
            vec![port("buck_content", "String")],
            Buck2GraphOp::Buck2(Buck2Op::GenerateBuckTargets),
        ),
        &extract_deps,
    )?;

    // Node: PrepareFileWrite (primitive - PURE) - generation 3
    let prepare_file_write = builder.add_node_after(
        Node::opaque(
            "prepare_file_write",
            vec![
                port("content", "String"),
                port("output_path", "String"),
            ],
            vec![port("request", "TransportRequest")],
            Buck2GraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &generate_targets,
    )?;

    // Node: ExecuteTransport (from gunbc-ops/transport - BOUNDARY) - generation 4
    let execute_transport = builder.add_node_after(
        Node::opaque(
            "execute_transport",
            vec![port("request", "TransportRequest")],
            vec![
                port("response", "TransportResponse"),
                port("written_path", "String"),
                port("content", "String"),
            ],
            Buck2GraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_file_write,
    )?;

    // Wire up the pipeline
    builder.add_edge(parse_cargo_toml.out("cargo_toml"), extract_deps.in_port("cargo_toml"))?;
    builder.add_edge(extract_deps.out("members"), generate_targets.in_port("members"))?;
    builder.add_edge(extract_deps.out("deps"), generate_targets.in_port("deps"))?;
    builder.add_edge(generate_targets.out("buck_content"), prepare_file_write.in_port("content"))?;
    builder.add_edge(prepare_file_write.out("request"), execute_transport.in_port("request"))?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_buck2_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 5);
        assert_eq!(dag.edges.len(), 5);
    }

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_buck2_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // ExecuteTransport should be the only boundary
        assert_eq!(boundaries.boundary_nodes.len(), 1);
        assert!(boundaries.is_boundary_node(&"execute_transport".into()));
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_buck2_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // cargo_toml_path and output_path are entrypoints
        assert!(entrypoints.is_entrypoint_port(&"parse_cargo_toml".into(), &"cargo_toml_path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_file_write".into(), &"output_path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_buck2_graph().expect("graph should build");

        assert_eq!(dag.nodes.len(), 5);
        assert_eq!(dag.edges.len(), 5);
    }

    #[test]
    fn test_intermediate_nodes_not_boundaries() {
        let dag = build_buck2_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        assert!(!boundaries.is_boundary_node(&"parse_cargo_toml".into()));
        assert!(!boundaries.is_boundary_node(&"extract_deps".into()));
        assert!(!boundaries.is_boundary_node(&"generate_targets".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_file_write".into()));
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_buck2_graph().expect("graph should build");
        let sig = buck2_signature();
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_buck2_graph().expect("graph should build");
        let inferred = infer_signature(&dag);
        
        assert_eq!(inferred.inputs.len(), 2);
        assert_eq!(inferred.outputs.len(), 3);
    }
}
