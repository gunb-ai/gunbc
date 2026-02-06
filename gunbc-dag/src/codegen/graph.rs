//! Graph builder for the codegen prep tool.
//!
//! Pipeline:
//! ```text
//! PrepareCodegenExists -> Execute -> ParseCodegenExists
//!                            |
//!                            v
//! PrepareCodegenCommand -> Execute -> ParseCodegenResult
//!                            |
//!                            v
//! PrepareStampWrite -> Execute
//! ```

use crate::codegen::ops::CodegenOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_ir::resource::ExecMode;
use gunbc_lib_transport::TransportOps;
use std::collections::HashMap;

/// Union type for codegen graph operations.
#[derive(Debug, Clone)]
pub enum CodegenGraphOp {
    /// Codegen-specific pure operations.
    Codegen(CodegenOp),
    /// Transport operations (boundary - actual I/O).
    Transport(TransportOps),
}

impl Executable for CodegenGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CodegenGraphOp::Codegen(op) => op.execute(inputs),
            CodegenGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Get the declared signature for the codegen workflow.
pub fn codegen_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // No inputs (all paths are derived from registry)
        // Outputs: parse result info + final stamp transport response
        .with_output("codegen_ran", "Bool", Cardinality::ONE)
        .with_output("prep_message", "String", Cardinality::ONE)
        .with_output("response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("skip", "Bool", Cardinality::ONE)
}

/// Build the codegen prep graph.
pub fn build_codegen_graph() -> Result<Dag<CodegenGraphOp>, BuilderError> {
    build_codegen_graph_with_mode(ExecMode::Ensure)
}

/// Build the codegen prep graph with a specific resource mode.
pub fn build_codegen_graph_with_mode(mode: ExecMode) -> Result<Dag<CodegenGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // Exists check stage
    // ========================================================================

    let prepare_codegen_exists = builder.add_root_node(Node::opaque(
        "prepare_codegen_exists",
        vec![],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        CodegenGraphOp::Codegen(CodegenOp::PrepareCodegenExists),
    ))?;

    let execute_codegen_exists = builder.add_node_after(
        Node::opaque(
            "execute_codegen_exists",
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![port("response", "TransportResponse")],
            CodegenGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_codegen_exists,
    )?;

    let parse_codegen_exists = builder.add_node_after(
        Node::opaque(
            "parse_codegen_exists",
            vec![port("response", "TransportResponse")],
            vec![port("codegen_needed", "Bool")],
            CodegenGraphOp::Codegen(CodegenOp::ParseCodegenExists(mode)),
        ),
        &execute_codegen_exists,
    )?;

    // ========================================================================
    // Codegen command stage
    // ========================================================================

    let prepare_codegen_command = builder.add_node_after(
        Node::opaque(
            "prepare_codegen_command",
            vec![port("codegen_needed", "Bool")],
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            CodegenGraphOp::Codegen(CodegenOp::PrepareCodegenCommand),
        ),
        &parse_codegen_exists,
    )?;

    let execute_codegen = builder.add_node_after(
        Node::opaque(
            "execute_codegen",
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
            ],
            CodegenGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_codegen_command,
    )?;

    let parse_codegen_result = builder.add_node_after(
        Node::opaque(
            "parse_codegen_result",
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
            ],
            vec![
                port("prep_success", "Bool"),
                port("codegen_ran", "Bool"),
                port("prep_message", "String"),
            ],
            CodegenGraphOp::Codegen(CodegenOp::ParseCodegenResult),
        ),
        &execute_codegen,
    )?;

    // ========================================================================
    // Stamp write stage
    // ========================================================================

    let prepare_stamp_write = builder.add_node_after(
        Node::opaque(
            "prepare_stamp_write",
            vec![port("prep_success", "Bool")],
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            CodegenGraphOp::Codegen(CodegenOp::PrepareStampWrite),
        ),
        &parse_codegen_result,
    )?;

    let execute_stamp_write = builder.add_node_after(
        Node::opaque(
            "execute_stamp_write",
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
            ],
            CodegenGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_stamp_write,
    )?;

    // ========================================================================
    // Wire up edges
    // ========================================================================

    builder.add_edge(
        prepare_codegen_exists.out("request"),
        execute_codegen_exists.in_port("request"),
    )?;
    builder.add_edge(
        prepare_codegen_exists.out("skip"),
        execute_codegen_exists.in_port("skip"),
    )?;
    builder.add_edge(
        execute_codegen_exists.out("response"),
        parse_codegen_exists.in_port("response"),
    )?;

    builder.add_edge(
        parse_codegen_exists.out("codegen_needed"),
        prepare_codegen_command.in_port("codegen_needed"),
    )?;

    builder.add_edge(
        prepare_codegen_command.out("request"),
        execute_codegen.in_port("request"),
    )?;
    builder.add_edge(
        prepare_codegen_command.out("skip"),
        execute_codegen.in_port("skip"),
    )?;

    builder.add_edge(
        execute_codegen.out("response"),
        parse_codegen_result.in_port("response"),
    )?;
    builder.add_edge(
        execute_codegen.out("skip"),
        parse_codegen_result.in_port("skip"),
    )?;

    builder.add_edge(
        parse_codegen_result.out("prep_success"),
        prepare_stamp_write.in_port("prep_success"),
    )?;

    builder.add_edge(
        prepare_stamp_write.out("request"),
        execute_stamp_write.in_port("request"),
    )?;
    builder.add_edge(
        prepare_stamp_write.out("skip"),
        execute_stamp_write.in_port("skip"),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_codegen_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 8);
        assert_eq!(dag.edges.len(), 10);
    }

    #[test]
    fn test_graph_has_transport_nodes() {
        let dag = build_codegen_graph().expect("graph should build");
        let transport_nodes: Vec<_> = dag
            .nodes
            .iter()
            .filter(|n| n.id.0.starts_with("execute_"))
            .collect();
        assert_eq!(transport_nodes.len(), 3);
    }

    #[test]
    fn test_graph_has_boundaries() {
        let dag = build_codegen_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);
        assert!(boundaries.is_boundary_node(&"parse_codegen_result".into()));
        assert!(boundaries.is_boundary_node(&"execute_stamp_write".into()));
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_codegen_graph().expect("graph should build");
        let sig = codegen_signature();
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_codegen_graph().expect("graph should build");
        let inferred = infer_signature(&dag);
        assert_eq!(inferred.inputs.len(), 0);
        assert_eq!(inferred.outputs.len(), 4);
    }
}
