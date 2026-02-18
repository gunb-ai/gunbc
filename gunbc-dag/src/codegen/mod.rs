//! gunbc-dag Codegen module.
//!
//! Upsert-style workflow for generating CLI entrypoints.

pub mod graph;
pub mod ops;

pub use crate::dsl_builder::build_codegen_graph_dsl;
pub use graph::{
    build_codegen_graph, build_codegen_graph_with_mode, codegen_signature, CodegenGraphOp,
};
pub use gunbc_ir::CODEGEN_STAMP_PATH;
pub use ops::CodegenOp;
