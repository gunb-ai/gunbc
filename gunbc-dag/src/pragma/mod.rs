//! gunbc-dag Pragma module.
//!
//! Pragma tool for generating clippy.toml and pragma allowlists.

pub mod ops;

use crate::dsl_builder::build_pragma_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

pub use ops::PragmaOp;

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

// ============================================================================
// Tool Target Registration
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "pragma",
    crate_name = "gunbc-pragma",
    description = "Generate clippy pragmas and lint policy",
    builder = "build_pragma_graph",
    import = "use gunbc_pragma::build_pragma_graph;",
    package = "dag",
    dsl_module = "pragma",
    outputs = "clippy.toml,tools/disallowed-methods-allowlist.txt,tools/pragma-lint-policy.txt",
    has_invocation,
    returns_result
)]
pub fn pragma_tool() {}
