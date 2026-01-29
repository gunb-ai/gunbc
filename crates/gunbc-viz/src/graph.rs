//! Graph builder for the visualization tool.

use crate::ops::VizOp;
use gunbc_ir::{build::*, Dag, Edge, Node};

/// Build the visualization graph.
///
/// Pipeline:
/// ```text
/// CollectDags -> ExportJson -> PrepareFileWrite -> ExecuteTransport
///                                                        ↓
///                                                   (boundary)
/// ```
pub fn build_viz_graph() -> Dag<VizOp> {
    let mut dag = Dag::new();

    // Node: CollectDags
    dag.add_node(Node::opaque(
        "collect_dags",
        vec![],
        vec![
            port("graph_count", "Int"),
            port("graph_names", "StrList"),
            port("graphs", "Json"),
        ],
        VizOp::CollectDags,
    ));

    // Node: ExportJson
    dag.add_node(Node::opaque(
        "export_json",
        vec![port("graphs", "Json")],
        vec![port("json_content", "String")],
        VizOp::ExportJson,
    ));

    // Node: PrepareFileWrite (PURE - no I/O)
    dag.add_node(Node::opaque(
        "prepare_file_write",
        vec![
            port("json_content", "String"),
            port("output_path", "String"),
        ],
        vec![port("request", "TransportRequest")],
        VizOp::PrepareFileWrite,
    ));

    // Node: ExecuteTransport (BOUNDARY - world write)
    dag.add_node(Node::opaque(
        "execute_transport",
        vec![port("request", "TransportRequest")],
        vec![
            port("response", "TransportResponse"),
            port("written_path", "String"),
        ],
        VizOp::ExecuteTransport,
    ));

    // Wire up the pipeline
    dag.add_edge(Edge::new("collect_dags", "graphs", "export_json", "graphs"));
    dag.add_edge(Edge::new(
        "export_json",
        "json_content",
        "prepare_file_write",
        "json_content",
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
        let dag = build_viz_graph();
        let boundaries = detect_boundaries(&dag);

        // ExecuteTransport should be a boundary
        assert!(boundaries.is_boundary_node(&"execute_transport".into()));
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_viz_graph();
        let entrypoints = detect_entrypoints(&dag);

        // output_path on prepare_file_write is an entrypoint
        assert!(entrypoints.is_entrypoint_port(&"prepare_file_write".into(), &"output_path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_viz_graph();

        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 3);
    }
}
