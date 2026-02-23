//! gunbc-dag Codegen module.
//!
//! Upsert-style workflow for generating CLI entrypoints.

pub mod graph;
pub mod ops;

pub use graph::{build_codegen_graph, codegen_signature, CodegenGraphOp};
pub use gunbc_ir::CODEGEN_STAMP_PATH;
pub use ops::CodegenOp;

// ============================================================================
// Tool Target Registration
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "codegen",
    crate_name = "gunbc-codegen",
    description = "Generate CLI entrypoints from tool registry",
    builder = "build_codegen_graph",
    dsl_module = "codegen",
    outputs = "target/codegen/.stamp",
    returns_result
)]
pub fn codegen_tool() {}
