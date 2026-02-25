//! DSL-backed infra orchestration graph.

use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for infra graphs.
pub type InfraGraphOp = DynOp;

/// Get the declared signature for the infra workflow (auto-derived from DAG).
pub fn build_signature() -> Result<WorkflowSignature, BuilderError> {
    build_infra_graph().map(|dag| infer_signature(&dag))
}

/// Build the infra orchestration graph from DSL source.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "infra",
    builder = "crate::infra::build_infra_graph().unwrap()"
)]
pub fn build_infra_graph() -> Result<Dag<InfraGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph("tools/infra.dag")
}
