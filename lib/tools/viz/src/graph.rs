//! Graph builder for the visualization tool.
//!
//! Composes viz-specific ops with library ops from lib crates.

use crate::ops::VizOp;
use gunbc_ir::{build::*, Dag, Edge, Node};
// Note: We use VizOp::PrepareVizOutput instead of FsOp::PrepareFileWrite
// to have a viz-specific default (viz-data.json)
use gunbc_lib_transport::TransportOps;

/// The operation type for viz graphs - a union of library and tool-specific ops.
#[derive(Debug, Clone)]
pub enum VizGraphOp {
    /// Viz-specific operations
    Viz(VizOp),
    /// Transport operations (from gunbc-ops)
    Transport(TransportOps),
}

impl gunbc_exec::Executable for VizGraphOp {
    fn execute(
        &self,
        inputs: std::collections::HashMap<String, gunbc_ir::Value>,
    ) -> Result<std::collections::HashMap<String, gunbc_ir::Value>, gunbc_exec::ExecError> {
        match self {
            VizGraphOp::Viz(op) => op.execute(inputs),
            VizGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Build the visualization graph.
///
/// Pipeline:
/// ```text
/// CollectDags -> ExportJson -> PrepareFileWrite -> ExecuteTransport
///    (viz)        (viz)            (fs)             (transport)
/// ```
pub fn build_viz_graph() -> Dag<VizGraphOp> {
    let mut dag = Dag::new();

    // Node: CollectDags (viz-specific)
    dag.add_node(Node::opaque(
        "collect_dags",
        vec![],
        vec![
            port("graph_count", "Int"),
            port("graph_names", "StrList"),
            port("graphs", "Json"),
        ],
        VizGraphOp::Viz(VizOp::CollectDags),
    ));

    // Node: ExportJson (viz-specific)
    dag.add_node(Node::opaque(
        "export_json",
        vec![port("graphs", "Json")],
        vec![
            port("json_content", "String"),
            port("content", "String"),  // Alias for FsOp compatibility
        ],
        VizGraphOp::Viz(VizOp::ExportJson),
    ));

    // Node: PrepareVizOutput (viz-specific - PURE)
    // Uses VizOp::PrepareVizOutput to have viz-specific default (viz-data.json)
    dag.add_node(Node::opaque(
        "prepare_file_write",
        vec![
            port("content", "String"),
            port("output_path", "String"),
        ],
        vec![port("request", "TransportRequest")],
        VizGraphOp::Viz(VizOp::PrepareVizOutput),
    ));

    // Node: ExecuteTransport (from gunbc-ops/transport - BOUNDARY)
    dag.add_node(Node::opaque(
        "execute_transport",
        vec![port("request", "TransportRequest")],
        vec![
            port("response", "TransportResponse"),
            port("written_path", "String"),
        ],
        VizGraphOp::Transport(TransportOps::Execute),
    ));

    // Wire up the pipeline
    dag.add_edge(Edge::new("collect_dags", "graphs", "export_json", "graphs"));
    dag.add_edge(Edge::new(
        "export_json",
        "content",  // Use the "content" alias
        "prepare_file_write",
        "content",
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
