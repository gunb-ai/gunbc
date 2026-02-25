//! gunbc-dag Pragma module.
//!
//! Pragma tool for generating clippy.toml and pragma allowlists.

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
    crate::dsl_builder::build_dsl_graph_for_entrypoint("tools/pragma.dag", Some("pragma"))
}
