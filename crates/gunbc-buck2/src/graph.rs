//! Graph builder for the Buck2 tool.

use crate::ops::Buck2Op;
use gunbc_ir::{build::*, Dag, Edge, Node};

/// Build the Buck2 generation graph.
///
/// Pipeline:
/// ```text
/// ParseCargoToml -> ExtractDeps -> GenerateBuckTargets -> PrepareFileWrite -> ExecuteTransport
///                                                                                    ↓
///                                                                               (boundary)
/// ```
///
/// The transport layer separates pure business logic (PrepareFileWrite) from I/O
/// (ExecuteTransport). The boundary is now at the transport level, making dry-run
/// interception uniform across all I/O operations.
pub fn build_buck2_graph() -> Dag<Buck2Op> {
    let mut dag = Dag::new();

    // Node: ParseCargoToml
    dag.add_node(Node::opaque(
        "parse_cargo_toml",
        vec![port("cargo_toml_path", "String")],
        vec![port("cargo_toml", "Json")],
        Buck2Op::ParseCargoToml,
    ));

    // Node: ExtractDeps
    dag.add_node(Node::opaque(
        "extract_deps",
        vec![port("cargo_toml", "Json")],
        vec![port("members", "StrList"), port("deps", "MapStrStr")],
        Buck2Op::ExtractDeps,
    ));

    // Node: GenerateBuckTargets
    dag.add_node(Node::opaque(
        "generate_targets",
        vec![port("members", "StrList"), port("deps", "MapStrStr")],
        vec![port("buck_content", "String")],
        Buck2Op::GenerateBuckTargets,
    ));

    // Node: PrepareFileWrite (PURE - no I/O)
    dag.add_node(Node::opaque(
        "prepare_file_write",
        vec![
            port("buck_content", "String"),
            port("output_path", "String"),
        ],
        vec![port("request", "TransportRequest")],
        Buck2Op::PrepareFileWrite,
    ));

    // Node: ExecuteTransport (BOUNDARY - world write)
    dag.add_node(Node::opaque(
        "execute_transport",
        vec![port("request", "TransportRequest")],
        vec![
            port("response", "TransportResponse"),
            port("written_path", "String"),
            port("content", "String"),
        ],
        Buck2Op::ExecuteTransport,
    ));

    // Wire up the pipeline
    dag.add_edge(Edge::new(
        "parse_cargo_toml",
        "cargo_toml",
        "extract_deps",
        "cargo_toml",
    ));
    dag.add_edge(Edge::new(
        "extract_deps",
        "members",
        "generate_targets",
        "members",
    ));
    dag.add_edge(Edge::new("extract_deps", "deps", "generate_targets", "deps"));
    dag.add_edge(Edge::new(
        "generate_targets",
        "buck_content",
        "prepare_file_write",
        "buck_content",
    ));
    dag.add_edge(Edge::new(
        "prepare_file_write",
        "request",
        "execute_transport",
        "request",
    ));

    dag
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_buck2_graph();
        let boundaries = detect_boundaries(&dag);

        // ExecuteTransport should be the only boundary
        assert_eq!(boundaries.boundary_nodes.len(), 1);
        assert!(boundaries.is_boundary_node(&"execute_transport".into()));
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_buck2_graph();
        let entrypoints = detect_entrypoints(&dag);

        // cargo_toml_path and output_path are entrypoints
        assert!(entrypoints.is_entrypoint_port(&"parse_cargo_toml".into(), &"cargo_toml_path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_file_write".into(), &"output_path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_buck2_graph();

        // Should have 5 nodes (added prepare_file_write and execute_transport)
        assert_eq!(dag.nodes.len(), 5);

        // Should have 5 edges
        assert_eq!(dag.edges.len(), 5);
    }

    #[test]
    fn test_intermediate_nodes_not_boundaries() {
        let dag = build_buck2_graph();
        let boundaries = detect_boundaries(&dag);

        // Intermediate nodes should not be boundaries
        assert!(!boundaries.is_boundary_node(&"parse_cargo_toml".into()));
        assert!(!boundaries.is_boundary_node(&"extract_deps".into()));
        assert!(!boundaries.is_boundary_node(&"generate_targets".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_file_write".into()));
    }
}
