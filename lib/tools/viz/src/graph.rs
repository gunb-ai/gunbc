//! Graph builder for the visualization tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.

use crate::ops::VizOp;
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, WorkflowSignature,
};
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

/// Get the declared signature for the viz workflow.
pub fn viz_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("output_path", "String", Cardinality::One)
        // Outputs - boundary outputs from execute_transport and intermediate nodes
        .with_output("graph_count", "Int", Cardinality::One)
        .with_output("graph_names", "StrList", Cardinality::One)
        .with_output("json_content", "String", Cardinality::One)
        .with_output("response", "TransportResponse", Cardinality::One)
        .with_output("written_path", "String", Cardinality::One)
}

/// Build the visualization graph using DagBuilder.
///
/// Pipeline:
/// ```text
/// CollectDags -> ExportJson -> PrepareFileWrite -> ExecuteTransport
///    (viz)        (viz)            (viz)             (transport)
/// ```
pub fn build_viz_graph() -> Result<Dag<VizGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: CollectDags (viz-specific) - generation 0
    let collect_dags = builder.add_root_node(Node::opaque(
        "collect_dags",
        vec![],
        vec![
            port("graph_count", "Int"),
            port("graph_names", "StrList"),
            port("graphs", "Json"),
        ],
        VizGraphOp::Viz(VizOp::CollectDags),
    ))?;

    // Node: ExportJson (viz-specific) - generation 1
    let export_json = builder.add_node_after(
        Node::opaque(
            "export_json",
            vec![port("graphs", "Json")],
            vec![
                port("json_content", "String"),
                port("content", "String"),
            ],
            VizGraphOp::Viz(VizOp::ExportJson),
        ),
        &collect_dags,
    )?;

    // Node: PrepareVizOutput (viz-specific - PURE) - generation 2
    let prepare_file_write = builder.add_node_after(
        Node::opaque(
            "prepare_file_write",
            vec![
                port("content", "String"),
                port("output_path", "String"),
            ],
            vec![port("request", "TransportRequest")],
            VizGraphOp::Viz(VizOp::PrepareVizOutput),
        ),
        &export_json,
    )?;

    // Node: ExecuteTransport (from gunbc-ops/transport - BOUNDARY) - generation 3
    let execute_transport = builder.add_node_after(
        Node::opaque(
            "execute_transport",
            vec![port("request", "TransportRequest")],
            vec![
                port("response", "TransportResponse"),
                port("written_path", "String"),
            ],
            VizGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_file_write,
    )?;

    // Wire up the pipeline
    builder.add_edge(collect_dags.out("graphs"), export_json.in_port("graphs"))?;
    builder.add_edge(export_json.out("content"), prepare_file_write.in_port("content"))?;
    builder.add_edge(prepare_file_write.out("request"), execute_transport.in_port("request"))?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_viz_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 3);
    }

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_viz_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        assert!(boundaries.is_boundary_node(&"execute_transport".into()));
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_viz_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        assert!(entrypoints.is_entrypoint_port(&"prepare_file_write".into(), &"output_path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_viz_graph().expect("graph should build");

        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 3);
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_viz_graph().expect("graph should build");
        let sig = viz_signature();
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_viz_graph().expect("graph should build");
        let inferred = infer_signature(&dag);
        
        // 1 input (output_path), 5 boundary outputs
        assert_eq!(inferred.inputs.len(), 1);
        assert_eq!(inferred.outputs.len(), 5);
    }
}
