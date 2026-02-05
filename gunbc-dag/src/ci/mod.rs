//! gunbc-dag CI module.
//!
//! CI orchestration for the gunbc repo.

pub mod env;
pub mod graph;
pub mod ops;

pub mod graph_mock;

pub use env::{mock_env_outputs, EnvOp};
pub use graph::{
    build_ci_graph, build_ci_graph_with_mode, ci_integrations, ci_signature, ci_workflow_config,
    ci_workflow_permissions, CIGraphOp,
};
pub use gunbc_ir::transport::github_actions::WorkflowConfig;
pub use gunbc_primitives::EmbeddedFileExistsOp;
pub use ops::CIOp;

#[cfg(test)]
mod generated_tests {
    #![allow(unused_imports)]
    include!("generated_tests.rs");
}
