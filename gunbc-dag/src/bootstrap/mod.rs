//! gunbc-dag Bootstrap module.
//!
//! Bootstrap tool for initializing gunbc projects.

pub mod graph;
pub mod ops;

pub mod graph_mock;

pub use graph::{build_bootstrap_graph, bootstrap_signature, BootstrapGraphOp};
pub use ops::BootstrapOp;

#[cfg(test)]
mod generated_tests {
    #![allow(unused_imports)]
    fn mock_spec() -> gunbc_test::MockSpec {
        crate::bootstrap::graph_mock::bootstrap_mock_spec()
    }
    include!("generated_tests.rs");
}
