//! gunbc-dag Codegen module — thin delegate to `dsl_builder::build_tool_graph`.

pub use gunbc_ir::CODEGEN_STAMP_PATH;

use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag, WorkflowSignature};

pub type CodegenGraphOp = DynOp;

pub fn codegen_signature() -> WorkflowSignature {
    crate::dsl_builder::tool_signature("codegen").unwrap_or_default()
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "codegen",
    builder = "build_codegen_graph().unwrap()"
)]
pub fn build_codegen_graph() -> Result<Dag<CodegenGraphOp>, BuilderError> {
    crate::dsl_builder::build_tool_graph("codegen")
}
