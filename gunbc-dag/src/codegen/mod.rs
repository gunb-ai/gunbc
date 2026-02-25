//! gunbc-dag Codegen module.
//!
//! Upsert-style workflow for generating CLI entrypoints.

pub use gunbc_ir::CODEGEN_STAMP_PATH;

use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for codegen graphs.
pub type CodegenGraphOp = DynOp;

/// Get the declared signature for the codegen workflow (auto-derived from DAG).
pub fn codegen_signature() -> WorkflowSignature {
    match build_codegen_graph() {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build codegen DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
}

/// Build the codegen graph from the DSL source.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "codegen",
    builder = "build_codegen_graph().unwrap()"
)]
pub fn build_codegen_graph() -> Result<Dag<CodegenGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph("tools/codegen.dag")
}
