//! gunbc-dag Pragma module.
//!
//! Pragma tool for generating clippy.toml and pragma allowlists.

pub mod graph;
pub mod ops;

pub use graph::{build_pragma_graph, pragma_signature, PragmaGraphOp};
pub use ops::PragmaOp;

// ============================================================================
// Tool Target Registration
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "pragma",
    crate_name = "gunbc-pragma",
    description = "Generate clippy pragmas and lint policy",
    builder = "build_pragma_graph",
    dsl_module = "pragma",
    returns_result
)]
pub fn pragma_tool() {}
