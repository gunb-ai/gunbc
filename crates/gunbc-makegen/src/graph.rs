//! Graph builder for the makegen tool.

use crate::ops::MakegenOp;
use gunbc_ir::{build::*, Dag, Edge, Node};

/// Build the makegen graph.
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
/// - `output_path`: One (optional, defaults to "Makefile")
/// - `written_path`, `content`: One (results)
/// - `changed`: One (boolean flag)
pub fn build_makegen_graph() -> Dag<MakegenOp> {
    let mut dag = Dag::new();

    // Node: LoadRegistry
    // No inputs (uses default registry)
    // Outputs: tool metadata and registry JSON
    dag.add_node(Node::opaque(
        "load_registry",
        vec![],
        vec![
            scalar("tool_count", "Int"),
            non_empty_list("tool_names", "StrList"),
            scalar("registry", "Json"),
        ],
        MakegenOp::LoadRegistry,
    ));

    // Node: RenderMakefile
    // Input: registry JSON
    // Output: generated Makefile content
    dag.add_node(Node::opaque(
        "render_makefile",
        vec![scalar("registry", "Json")],
        vec![scalar("makefile_content", "String")],
        MakegenOp::RenderMakefile,
    ));

    // Node: WriteMakefile (BOUNDARY - world write)
    // Input: content and optional path
    // Output: write results
    dag.add_node(Node::opaque(
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
    ));

    // Wire up the pipeline
    dag.add_edge(Edge::new("load_registry", "registry", "render_makefile", "registry"));
    dag.add_edge(Edge::new("render_makefile", "makefile_content", "write_makefile", "makefile_content"));

    dag
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_makegen_graph();
        let boundaries = detect_boundaries(&dag);

        // WriteMakefile should be a boundary (world write)
        assert!(boundaries.is_boundary_node(&"write_makefile".into()));
        // load_registry also has unconnected outputs (tool_count, tool_names) 
        // which are informational - that's fine, they're secondary boundaries
        assert!(boundaries.boundary_nodes.len() >= 1);
    }

    #[test]
    fn test_graph_has_entrypoint() {
        let dag = build_makegen_graph();
        let entrypoints = detect_entrypoints(&dag);

        // output_path is an entrypoint (input to write_makefile with no upstream)
        assert!(entrypoints.is_entrypoint_port(&"write_makefile".into(), &"output_path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_makegen_graph();

        assert_eq!(dag.nodes.len(), 3);
        assert_eq!(dag.edges.len(), 2);
    }
}
