//! DSL-backed infra orchestration graph — thin delegate to `dsl_builder::build_tool_graph`.

use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag, WorkflowSignature};

pub type InfraGraphOp = DynOp;

pub fn build_signature() -> Result<WorkflowSignature, BuilderError> {
    crate::dsl_builder::tool_signature("infra")
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "infra",
    builder = "crate::infra::build_infra_graph().unwrap()"
)]
pub fn build_infra_graph() -> Result<Dag<InfraGraphOp>, BuilderError> {
    crate::dsl_builder::build_tool_graph("infra")
}
