//! gunbc-dag Bootstrap module.
//!
//! Bootstrap tool for initializing gunbc projects.

pub mod ops;

use crate::dsl_builder::build_bootstrap_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

pub use ops::BootstrapOp;

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
    build_bootstrap_graph_dsl()
}

// ============================================================================
// Tool Target Registrations
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "bootstrap",
    crate_name = "gunbc-bootstrap",
    description = "Generate Makefile and .gitignore",
    builder = "build_bootstrap_graph",
    import = "use gunbc_bootstrap::build_bootstrap_graph;",
    package = "dag",
    binary = "bootstrap",
    dsl_module = "bootstrap",
    outputs = "Makefile,.gitignore",
    has_invocation,
    returns_result
)]
pub fn bootstrap_tool() {}
