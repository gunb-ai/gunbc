//! DSL-backed graph builder for the bootstrap tool.

use crate::dsl_builder::build_bootstrap_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for bootstrap graphs.
pub type BootstrapGraphOp = DynOp;

/// Get the declared signature for the bootstrap workflow (auto-derived from DAG).
pub fn bootstrap_signature() -> WorkflowSignature {
    infer_signature(&build_bootstrap_graph().expect("bootstrap DAG should build for signature"))
}

/// Build bootstrap graph from the DSL source.
pub fn build_bootstrap_graph() -> Result<Dag<BootstrapGraphOp>, BuilderError> {
    build_bootstrap_graph_dsl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_bootstrap_graph_from_dsl() {
        let dag = build_bootstrap_graph().expect("bootstrap DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }

}
