//! DSL-backed deps tool (replaces lib/tools/deps/src/graph.rs).

use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};

/// Runtime op type for deps graphs.
pub type DepsGraphOp = DynOp;

/// Build the deps graph from the DSL source.
pub fn build_deps_graph() -> Result<Dag<DepsGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph("tools/deps.dag")
}
