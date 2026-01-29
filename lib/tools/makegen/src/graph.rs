//! Graph builder for the makegen tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.

use crate::ops::MakegenOp;
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, WorkflowSignature,
};

/// Get the declared signature for the makegen workflow.
pub fn makegen_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("output_path", "String", Cardinality::ZeroOrOne)
        // Outputs - boundary outputs
        .with_output("tool_count", "Int", Cardinality::One)
        .with_output("tool_names", "StrList", Cardinality::OneOrMore)
        .with_output("written_path", "String", Cardinality::One)
        .with_output("content", "String", Cardinality::One)
        .with_output("changed", "Bool", Cardinality::One)
}

/// Build the makegen graph using DagBuilder.
///
/// Pipeline:
/// ```text
/// LoadRegistry -> RenderMakefile -> WriteMakefile
///                                        ↓
///                                   (boundary)
/// ```
///
/// # Port Cardinalities
///
/// - `tool_count`: One (scalar integer)
/// - `tool_names`: OneOrMore (at least one tool should exist)
/// - `registry`: One (JSON registry object)
/// - `makefile_content`: One (generated content)
/// - `output_path`: ZeroOrOne (optional, defaults to "Makefile")
/// - `written_path`, `content`: One (results)
/// - `changed`: One (boolean flag)
pub fn build_makegen_graph() -> Result<Dag<MakegenOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: LoadRegistry - generation 0
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
        MakegenOp::LoadRegistry,
    ))?;

    // Node: RenderMakefile - generation 1
    // Input: registry JSON
    // Output: generated Makefile content
    let render_makefile = builder.add_node_after(
        Node::opaque(
            "render_makefile",
            vec![scalar("registry", "Json")],
            vec![scalar("makefile_content", "String")],
            MakegenOp::RenderMakefile,
        ),
        &load_registry,
    )?;

    // Node: WriteMakefile (BOUNDARY - world write) - generation 2
    // Input: content and optional path
    // Output: write results
    let write_makefile = builder.add_node_after(
        Node::opaque(
            "write_makefile",
            vec![
                scalar("makefile_content", "String"),
                optional("output_path", "String"),
            ],
            vec![
                scalar("written_path", "String"),
                scalar("content", "String"),
                scalar("changed", "Bool"),
            ],
            MakegenOp::WriteMakefile,
        ),
        &render_makefile,
    )?;

    // Wire up the pipeline
    builder.add_edge(load_registry.out("registry"), render_makefile.in_port("registry"))?;
    builder.add_edge(render_makefile.out("makefile_content"), write_makefile.in_port("makefile_content"))?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_makegen_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 3);
        assert_eq!(dag.edges.len(), 2);
    }

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_makegen_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // WriteMakefile should be a boundary (world write)
        assert!(boundaries.is_boundary_node(&"write_makefile".into()));
        // load_registry also has unconnected outputs (tool_count, tool_names) 
        // which are informational - that's fine, they're secondary boundaries
        assert!(boundaries.boundary_nodes.len() >= 1);
    }

    #[test]
    fn test_graph_has_entrypoint() {
        let dag = build_makegen_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // output_path is an entrypoint (input to write_makefile with no upstream)
        assert!(entrypoints.is_entrypoint_port(&"write_makefile".into(), &"output_path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_makegen_graph().expect("graph should build");

        assert_eq!(dag.nodes.len(), 3);
        assert_eq!(dag.edges.len(), 2);
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
        
        // 1 input (output_path), 5 boundary outputs
        assert_eq!(inferred.inputs.len(), 1);
        assert_eq!(inferred.outputs.len(), 5);
    }
}
