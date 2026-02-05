//! gunbc-dag Bootstrap module.
//!
//! Bootstrap tool for initializing gunbc projects.

pub mod graph;
pub mod ops;

pub mod graph_mock;

pub use graph::{bootstrap_signature, build_bootstrap_graph, BootstrapGraphOp};
pub use ops::BootstrapOp;

#[cfg(test)]
mod generated_tests {
    #![allow(unused_imports)]
    include!("generated_tests.rs");
}
