//! DSL-backed graph builder for dag-viz.

use crate::dsl_builder::build_dag_viz_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Mode for the dag-viz CLI entrypoints.
#[derive(Debug, Clone)]
pub enum DagVizMode {
    Snapshot,
    Diff { base_ref: String },
    Recent,
    SaveSnapshot,
}

/// Runtime op type for dag-viz graphs.
pub type DagVizGraphOp = DynOp;

/// Get the declared signature for the dag-viz workflow (auto-derived from DAG).
pub fn dag_viz_signature(mode: &DagVizMode) -> WorkflowSignature {
    match build_dag_viz_graph(mode.clone()) {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build dag_viz DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
}

/// Build dag-viz graph from the canonical DSL source.
///
/// The DSL module models mode selection internally; the Rust mode argument is
/// retained for compatibility with existing tool registrations and tests.
pub fn build_dag_viz_graph(_mode: DagVizMode) -> Result<Dag<DagVizGraphOp>, BuilderError> {
    build_dag_viz_graph_dsl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_dag_viz_graph_from_dsl() {
        let dag = build_dag_viz_graph(DagVizMode::Snapshot).expect("dag_viz DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }
}
