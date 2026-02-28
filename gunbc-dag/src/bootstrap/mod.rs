//! gunbc-dag Bootstrap module — thin delegate to `dsl_builder::build_tool_graph`.

use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag, WorkflowSignature};

pub type BootstrapGraphOp = DynOp;

pub fn bootstrap_signature() -> WorkflowSignature {
    crate::dsl_builder::tool_signature("bootstrap").unwrap_or_default()
}

pub fn build_bootstrap_graph() -> Result<Dag<BootstrapGraphOp>, BuilderError> {
    crate::dsl_builder::build_tool_graph("bootstrap")
}
