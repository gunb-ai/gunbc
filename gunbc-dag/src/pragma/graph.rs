//! DSL-backed graph builder for the pragma tool.

use crate::dsl_builder::build_pragma_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for pragma graphs.
pub type PragmaGraphOp = DynOp;

/// Get the declared signature for the pragma workflow (auto-derived from DAG).
pub fn pragma_signature() -> WorkflowSignature {
    match build_pragma_graph() {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build pragma DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
}

/// Build pragma graph from the DSL source.
pub fn build_pragma_graph() -> Result<Dag<PragmaGraphOp>, BuilderError> {
    build_pragma_graph_dsl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_pragma_graph_from_dsl() {
        let dag = build_pragma_graph().expect("pragma DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }
}
