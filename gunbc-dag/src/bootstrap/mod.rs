//! gunbc-dag Bootstrap module.
//!
//! Bootstrap tool for initializing gunbc projects.

pub mod graph;
pub mod ops;

pub mod graph_mock;

pub use crate::dsl_builder::build_bootstrap_graph_dsl;
pub use graph::{bootstrap_signature, build_bootstrap_graph, BootstrapGraphOp};
pub use ops::BootstrapOp;

// ============================================================================
// Tool Target Registrations
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "bootstrap",
    crate_name = "gunbc-bootstrap",
    description = "Generate Makefile and .gitignore",
    builder = "build_bootstrap_graph",
    import = "use gunbc_bootstrap::build_bootstrap_graph;",
    mock_spec = "gunbc_dag::bootstrap::graph_mock::bootstrap_mock_spec()",
    package = "dag",
    binary = "bootstrap",
    dsl_module = "bootstrap",
    has_invocation,
    returns_result
)]
pub fn bootstrap_tool() {}

#[cfg(test)]
mod generated_tests {
    include!("generated_tests.rs");
}
