//! gunbc-dag doc generation module.
//!
//! Generates documentation with live code excerpts and test indices.

use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};

/// Runtime op type for docgen graphs.
pub type DocgenGraphOp = DynOp;

/// Build docgen graph from the DSL source.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "docgen",
    builder = "build_docgen_graph().unwrap()"
)]
pub fn build_docgen_graph() -> Result<Dag<DocgenGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph("tools/docgen.dag")
}
