//! gunbc-dag Bootstrap module.
//!
//! Bootstrap tool for initializing gunbc projects.

use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for bootstrap graphs.
pub type BootstrapGraphOp = DynOp;

/// Get the declared signature for the bootstrap workflow (auto-derived from DAG).
pub fn bootstrap_signature() -> WorkflowSignature {
    match build_bootstrap_graph() {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build bootstrap DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
}

/// Build bootstrap graph from the DSL source.
pub fn build_bootstrap_graph() -> Result<Dag<BootstrapGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph("tools/bootstrap.dag")
}
