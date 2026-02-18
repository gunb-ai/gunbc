//! gunbc-dag Pragma module.
//!
//! Pragma tool for generating clippy.toml and pragma allowlists.

pub mod graph;
pub mod ops;

pub mod graph_mock;

pub use crate::dsl_builder::build_pragma_graph_dsl;
pub use graph::{build_pragma_graph, pragma_signature, PragmaGraphOp};
pub use ops::PragmaOp;

#[cfg(test)]
#[allow(unused_variables)]
mod generated_tests {
    include!("generated_tests.rs");
}
