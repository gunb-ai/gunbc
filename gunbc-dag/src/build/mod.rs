//! gunbc-dag Build module.
//!
//! Local development build pipeline with DAG progress visualization.
//! Wraps cargo build, test, and clippy in a progress-tracked DAG.

use crate::dsl_builder::build_build_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for build graphs.
pub type BuildGraphOp = DynOp;

/// Get the declared signature for the build workflow (auto-derived from DAG).
pub fn build_signature() -> Result<WorkflowSignature, BuilderError> {
    build_build_graph().map(|dag| infer_signature(&dag))
}

/// Build the build graph from the DSL source.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "build",
    builder = "build_build_graph().unwrap()"
)]
pub fn build_build_graph() -> Result<Dag<BuildGraphOp>, BuilderError> {
    build_build_graph_dsl()
}
