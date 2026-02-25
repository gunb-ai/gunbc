//! gunbc-dag Codegen module.
//!
//! Upsert-style workflow for generating CLI entrypoints.

pub use gunbc_ir::CODEGEN_STAMP_PATH;

use crate::dsl_builder::build_codegen_graph_dsl;
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
    build_codegen_graph_dsl()
}

// ============================================================================
// Tool Target Registration
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "codegen",
    crate_name = "gunbc-dag",
    description = "Generate CLI entrypoints from tool registry",
    builder = "build_codegen_graph",
    import = "use gunbc_dag::build_codegen_graph;",
    mock_spec = r#"gunbc_dag::mock_defaults::auto_mock_spec(&dag, "codegen")"#,
    dsl_module = "codegen",
    outputs = "target/codegen/.stamp",
    provides = "target/codegen/.stamp",
    returns_result
)]
pub fn codegen_tool() {}
