//! gunbc-dag CI module.
//!
//! CI orchestration for the gunbc repo.

pub mod graph;
pub mod ops;

pub mod graph_mock;

pub use graph::{
    build_ci_graph, ci_integrations, ci_signature, ci_workflow_config, ci_workflow_permissions,
    CIGraphOp,
};
pub use gunbc_primitives::EmbeddedFileExistsOp;
pub use gunbc_ir::transport::github_actions::WorkflowConfig;
pub use ops::CIOp;

#[cfg(test)]
mod generated_tests {
    #![allow(unused_imports)]
    fn mock_spec() -> gunbc_test::MockSpec {
        crate::ci::graph_mock::ci_mock_spec()
    }
    include!("generated_tests.rs");
}
