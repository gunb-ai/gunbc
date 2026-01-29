//! Graph builder for the prep tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.

use crate::ops::PrepOp;
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, WorkflowSignature,
};

/// Get the declared signature for the prep workflow.
pub fn prep_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints) - dry_run is passed to multiple nodes
        .with_input("dry_run", "Bool", Cardinality::ZeroOrOne)
        // Outputs - boundary outputs (all outputs from nodes with no downstream)
        .with_output("build_ran", "Bool", Cardinality::One)
        .with_output("build_success", "Bool", Cardinality::One)
        // Additional outputs from intermediate nodes that are also boundaries
        .with_output("buck_out_exists", "Bool", Cardinality::One)
        .with_output("codegen_skipped", "Bool", Cardinality::One)
}

/// Build the prep graph using DagBuilder.
///
/// Pipeline:
/// ```text
/// CheckState -> RunCodegen -> RunDaggen -> Build
///                                           ↓
///                                       (boundary)
/// ```
///
/// # Port Cardinalities
///
/// - `needs_codegen`: One (boolean flag)
/// - `codegen_ran`, `daggen_ran`, `build_ran`: One (boolean flags)
/// - `build_success`: One (boolean result)
/// - `dry_run`: ZeroOrOne (optional flag)
pub fn build_prep_graph() -> Result<Dag<PrepOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: CheckState - generation 0
    // Check if codegen is needed
    let check_state = builder.add_root_node(Node::opaque(
        "check_state",
        vec![],
        vec![
            scalar("needs_codegen", "Bool"),
            scalar("buck_out_exists", "Bool"),
        ],
        PrepOp::CheckState,
    ))?;

    // Node: RunCodegen - generation 1
    // Run CLI generation if needed
    let run_codegen = builder.add_node_after(
        Node::opaque(
            "run_codegen",
            vec![
                scalar("needs_codegen", "Bool"),
                optional("dry_run", "Bool"),
            ],
            vec![
                scalar("codegen_ran", "Bool"),
                scalar("codegen_skipped", "Bool"),
            ],
            PrepOp::RunCodegen,
        ),
        &check_state,
    )?;

    // Node: RunDaggen - generation 2
    // Run graph.rs generation
    let run_daggen = builder.add_node_after(
        Node::opaque(
            "run_daggen",
            vec![
                scalar("codegen_ran", "Bool"),
                optional("dry_run", "Bool"),
            ],
            vec![scalar("daggen_ran", "Bool")],
            PrepOp::RunDaggen,
        ),
        &run_codegen,
    )?;

    // Node: Build (BOUNDARY) - generation 3
    // Build all targets
    let build = builder.add_node_after(
        Node::opaque(
            "build",
            vec![
                scalar("daggen_ran", "Bool"),
                optional("dry_run", "Bool"),
            ],
            vec![
                scalar("build_ran", "Bool"),
                scalar("build_success", "Bool"),
            ],
            PrepOp::Build,
        ),
        &run_daggen,
    )?;

    // Wire up the pipeline
    builder.add_edge(check_state.out("needs_codegen"), run_codegen.in_port("needs_codegen"))?;
    builder.add_edge(run_codegen.out("codegen_ran"), run_daggen.in_port("codegen_ran"))?;
    builder.add_edge(run_daggen.out("daggen_ran"), build.in_port("daggen_ran"))?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_prep_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 3);
    }

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_prep_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Build should be a boundary (it's the final step)
        assert!(boundaries.is_boundary_node(&"build".into()));
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_prep_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // dry_run inputs are entrypoints
        assert!(entrypoints.is_entrypoint_port(&"run_codegen".into(), &"dry_run".into()));
        assert!(entrypoints.is_entrypoint_port(&"run_daggen".into(), &"dry_run".into()));
        assert!(entrypoints.is_entrypoint_port(&"build".into(), &"dry_run".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_prep_graph().expect("graph should build");

        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 3);
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_prep_graph().expect("graph should build");
        let sig = prep_signature();
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_prep_graph().expect("graph should build");
        let inferred = infer_signature(&dag);
        
        // 3 dry_run inputs (one per node that accepts it), 4 boundary outputs
        // (check_state, run_codegen, run_daggen all have outputs that aren't consumed downstream)
        assert!(inferred.inputs.len() >= 1);
        assert_eq!(inferred.outputs.len(), 4);
    }
}
