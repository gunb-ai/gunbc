//! gunbc-dag Build module — thin delegate to `dsl_builder::build_tool_graph`.

use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag, WorkflowSignature};

pub type BuildGraphOp = DynOp;

pub fn build_signature() -> Result<WorkflowSignature, BuilderError> {
    crate::dsl_builder::tool_signature("build")
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "build",
    builder = "build_build_graph().unwrap()"
)]
pub fn build_build_graph() -> Result<Dag<BuildGraphOp>, BuilderError> {
    crate::dsl_builder::build_tool_graph("build")
}
