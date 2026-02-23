//! DSL-backed deps tool (replaces lib/tools/deps/src/graph.rs).

use crate::dsl_builder::build_deps_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};

/// Runtime op type for deps graphs.
pub type DepsGraphOp = DynOp;

/// Build the deps graph from the DSL source.
pub fn build_deps_graph() -> Result<Dag<DepsGraphOp>, BuilderError> {
    build_deps_graph_dsl()
}

// ============================================================================
// Tool Target Registration
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "deps",
    crate_name = "gunbc-deps",
    description = "Install tool dependencies",
    builder = "build_deps_graph",
    import = "use gunbc_dag::deps_tool::build_deps_graph;",
    package = "dag",
    binary = "deps",
    entrypoints = r#"[{"port_name":"manifest_path","type_id":"String","short":"m","help":"Path to deps.toml manifest","make_var":"MANIFEST"}]"#,
    dsl_module = "deps",
    outputs = "deps.toml",
    has_invocation,
    returns_result
)]
pub fn deps_tool() {}
