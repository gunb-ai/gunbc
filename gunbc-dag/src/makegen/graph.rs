//! Graph builder for the makegen tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! This tool follows the transport pattern:
//! - Pure ops prepare data and `TransportRequest` values
//! - `TransportOps::Execute` is the single boundary that does actual I/O

use crate::makegen::ops::MakegenOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrepareFileWriteOp;
use std::collections::HashMap;

/// The operation type for makegen graphs - a union of makegen ops, primitives, and transport.
#[derive(Debug, Clone)]
pub enum MakegenGraphOp {
    /// Makegen-specific operations
    Makegen(MakegenOp),
    /// Prepare file write (primitive - PURE)
    PrepareFileWrite(PrepareFileWriteOp),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl Executable for MakegenGraphOp {
    fn execute(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            MakegenGraphOp::Makegen(op) => op.execute(inputs),
            MakegenGraphOp::PrepareFileWrite(op) => op.execute(inputs),
            MakegenGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Get the declared signature for the makegen workflow.
pub fn makegen_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("output_path", "String", Cardinality::ZeroOrOne)
        // Outputs from execute_transport (boundary)
        .with_output("response", "TransportResponse", Cardinality::One)
        .with_output("written_path", "String", Cardinality::One)
        .with_output("content", "String", Cardinality::One)
        // Informational outputs from load_registry (secondary boundaries)
        .with_output("tool_count", "Int", Cardinality::One)
        .with_output("tool_names", "StrList", Cardinality::OneOrMore)
}

/// Build the makegen graph using DagBuilder.
///
/// Pipeline (follows transport pattern like Buck2):
/// ```text
/// LoadRegistry -> RenderMakefile -> PrepareFileWrite -> ExecuteTransport
///    (makegen)      (makegen)         (primitive)        (transport)
///                                        PURE              BOUNDARY
/// ```
///
/// # Port Cardinalities
///
/// - `tool_count`: One (scalar integer) - informational, not connected
/// - `tool_names`: OneOrMore (at least one tool should exist) - informational
/// - `registry`: One (JSON registry object)
/// - `makefile_content`: One (generated content)
/// - `output_path`: ZeroOrOne (optional, defaults to "Makefile")
/// - `request`: One (TransportRequest for file write)
/// - `response`, `written_path`, `content`: One (transport outputs)
#[allow(clippy::result_large_err)]
pub fn build_makegen_graph() -> Result<Dag<MakegenGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: LoadRegistry (makegen-specific) - generation 0
    // No inputs (uses default registry)
    // Outputs: tool metadata and registry JSON
    let load_registry = builder.add_root_node(Node::opaque(
        "load_registry",
        vec![],
        vec![
            scalar("tool_count", "Int"),
            non_empty_list("tool_names", "StrList"),
            scalar("registry", "Json"),
        ],
        MakegenGraphOp::Makegen(MakegenOp::LoadRegistry),
    ))?;

    // Node: RenderMakefile (makegen-specific) - generation 1
    // Input: registry JSON
    // Output: generated Makefile content
    let render_makefile = builder.add_node_after(
        Node::opaque(
            "render_makefile",
            vec![scalar("registry", "Json")],
            vec![scalar("makefile_content", "String")],
            MakegenGraphOp::Makegen(MakegenOp::RenderMakefile),
        ),
        &load_registry,
    )?;

    // Node: PrepareFileWrite (primitive - PURE) - generation 2
    // Input: content and optional path
    // Output: TransportRequest (no I/O happens here)
    let prepare_file_write = builder.add_node_after(
        Node::opaque(
            "prepare_file_write",
            vec![
                port("content", "String"),
                optional("output_path", "String"),
            ],
            vec![port("request", "TransportRequest")],
            MakegenGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &render_makefile,
    )?;

    // Node: ExecuteTransport (transport - BOUNDARY) - generation 3
    // Input: TransportRequest
    // Output: TransportResponse + extracted fields (actual I/O happens here)
    let execute_transport = builder.add_node_after(
        Node::opaque(
            "execute_transport",
            vec![port("request", "TransportRequest")],
            vec![
                port("response", "TransportResponse"),
                port("written_path", "String"),
                port("content", "String"),
            ],
            MakegenGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_file_write,
    )?;

    // Wire up the pipeline
    builder.add_edge(load_registry.out("registry"), render_makefile.in_port("registry"))?;
    builder.add_edge(render_makefile.out("makefile_content"), prepare_file_write.in_port("content"))?;
    builder.add_edge(prepare_file_write.out("request"), execute_transport.in_port("request"))?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_makegen_graph().expect("graph should build");
        // 4 nodes: LoadRegistry, RenderMakefile, PrepareFileWrite, ExecuteTransport
        assert_eq!(dag.nodes.len(), 4);
        // 3 edges: registry->render, content->prepare, request->execute
        assert_eq!(dag.edges.len(), 3);
    }

    #[test]
    fn test_graph_has_single_transport_boundary() {
        let dag = build_makegen_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // ExecuteTransport is the primary boundary (actual I/O)
        assert!(boundaries.is_boundary_node(&"execute_transport".into()));
        
        // load_registry also has unconnected outputs (tool_count, tool_names)
        // which are informational secondary boundaries - that's expected
    }

    #[test]
    fn test_graph_has_entrypoint() {
        let dag = build_makegen_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // output_path is an entrypoint (input to prepare_file_write with no upstream)
        assert!(entrypoints.is_entrypoint_port(&"prepare_file_write".into(), &"output_path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_makegen_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 3);
    }

    #[test]
    fn test_intermediate_nodes_not_boundaries() {
        let dag = build_makegen_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // PrepareFileWrite is NOT a boundary - it's pure
        assert!(!boundaries.is_boundary_node(&"prepare_file_write".into()));
        // RenderMakefile is NOT a boundary - all outputs connected
        assert!(!boundaries.is_boundary_node(&"render_makefile".into()));
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_makegen_graph().expect("graph should build");
        let sig = makegen_signature();
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_makegen_graph().expect("graph should build");
        let inferred = infer_signature(&dag);
        
        // 1 input (output_path)
        assert_eq!(inferred.inputs.len(), 1);
        // Boundary outputs: execute_transport (3) + load_registry informational (2)
        assert_eq!(inferred.outputs.len(), 5);
    }
}
