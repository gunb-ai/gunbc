//! DSL-backed graph builder for infra orchestration.

use crate::dsl_builder::build_infra_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for infra graphs.
pub type InfraGraphOp = DynOp;

/// Get the declared signature for the infra workflow (auto-derived from DAG).
pub fn build_signature() -> WorkflowSignature {
    match build_infra_graph() {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build infra DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
}

/// Build the infra orchestration graph from DSL source.
pub fn build_infra_graph() -> Result<Dag<InfraGraphOp>, BuilderError> {
    build_infra_graph_dsl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_infra_graph_from_dsl() {
        let dag = build_infra_graph().expect("infra DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }
}
