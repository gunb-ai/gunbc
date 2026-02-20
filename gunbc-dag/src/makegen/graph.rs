//! DSL-backed graph builder for the makegen tool.

use crate::dsl_builder::build_makegen_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for makegen graphs.
pub type MakegenGraphOp = DynOp;

/// Get the declared signature for the makegen workflow (auto-derived from DAG).
pub fn makegen_signature() -> WorkflowSignature {
    match build_makegen_graph() {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build makegen DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
}

/// Build makegen graph from the DSL source.
pub fn build_makegen_graph() -> Result<Dag<MakegenGraphOp>, BuilderError> {
    build_makegen_graph_dsl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_makegen_graph_from_dsl() {
        let dag = build_makegen_graph().expect("makegen DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }
}
