//! Composable operation patterns for common DAG structures.
//!
//! This module provides builders for common operation patterns:
//!
//! - [`UpsertBuilder`]: Check → Create → Resolve pattern for idempotent resource creation
//! - [`TransactionBuilder`]: Begin → Body → Commit/Rollback pattern for transactional operations
//! - [`AtomicBuilder`]: Precondition → Operation → Postcondition pattern for atomic operations
//! - [`LoopBuilder`]: Iterate over collection, applying body to each element
//! - [`BranchBuilder`]: Conditional execution based on boolean condition
//! - [`RetryBuilder`]: Re-execute on failure with configurable backoff
//! - [`WhileBuilder`]: Re-execute while a condition holds
//! - [`PollBuilder`]: Re-execute at intervals until success or timeout
//!
//! Each builder creates a [`Node`] with a [`NodeBody::SubDag`] containing the pattern's
//! internal structure, with proper guards for conditional execution.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::patterns::UpsertBuilder;
//!
//! let upsert_node = UpsertBuilder::new("install_tool")
//!     .with_check(MyOp::CheckInstalled)
//!     .with_create(MyOp::Install)
//!     .with_resolve(MyOp::Verify)
//!     .build();
//! ```

pub mod atomic;
pub mod authenticate;
pub mod branch;
pub mod content_upsert;
pub mod emit;
pub mod loop_pattern;
pub mod pattern_op;
pub mod repeat;
pub mod transaction;
pub mod transport_triplet;
pub mod upsert;

pub use atomic::AtomicBuilder;
pub use authenticate::{
    canonical_authenticate_chain, validate_authenticate_bindings, validate_authenticate_chain,
    AuthenticatePhase, AuthenticatePhaseBinding,
};
pub use branch::BranchBuilder;
pub use emit::EmitBuilder;
pub use loop_pattern::LoopBuilder;
pub use pattern_op::PatternOp;
pub use repeat::{
    BackoffStrategy, FailureClassifier, PollBuilder, RepeatPolicy, RetryBuilder, WhileBuilder,
};
pub use transaction::TransactionBuilder;
pub use upsert::UpsertBuilder;

/// A declared resource input for a pattern's body DAG.
///
/// When a pattern builder (Loop, Branch, Retry, etc.) delegates to a body DAG
/// that needs resources, the builder can declare which `res:*` ports the body
/// requires. At build time, we validate that every declared `ResourceInput`
/// has a matching entrypoint in the body DAG.
///
/// Without explicit declaration, resource wiring still works via auto-inference
/// (SubDag entrypoints bubble up). This type enables opt-in explicit validation
/// for teams that want stricter guarantees.
#[derive(Debug, Clone)]
pub struct ResourceInput {
    /// Port name (must start with "res:")
    pub port_name: String,
    /// Resource type identifier, e.g. "Platform", "Credential"
    pub type_id: String,
}

impl ResourceInput {
    /// Create a new resource input declaration.
    ///
    /// # Panics
    ///
    /// Panics if `port_name` does not start with `"res:"`.
    pub fn new(port_name: impl Into<String>, type_id: impl Into<String>) -> Self {
        let port_name = port_name.into();
        assert!(
            port_name.starts_with("res:"),
            "ResourceInput port_name must start with 'res:', got '{}'",
            port_name
        );
        Self {
            port_name,
            type_id: type_id.into(),
        }
    }
}

/// Validate that every declared `ResourceInput` has a matching entrypoint in the body DAG.
///
/// Used by pattern builders at build time.
///
/// # Panics
///
/// Panics with a descriptive message if a declared resource input has no matching
/// entrypoint in the body DAG.
pub(crate) fn validate_resource_inputs<T>(
    pattern_name: &str,
    resource_inputs: &[ResourceInput],
    body_dag: &crate::dag::Dag<T>,
) {
    validate_resource_inputs_any(pattern_name, resource_inputs, &[body_dag]);
}

/// Validate that every declared `ResourceInput` has a matching entrypoint in
/// at least one of the provided DAGs.
///
/// Used by patterns with multiple sub-DAGs (e.g., WhileBuilder has both a
/// condition DAG and a body DAG).
///
/// # Panics
///
/// Panics if a declared resource input has no matching entrypoint in any DAG.
pub(crate) fn validate_resource_inputs_any<T>(
    pattern_name: &str,
    resource_inputs: &[ResourceInput],
    dags: &[&crate::dag::Dag<T>],
) {
    for ri in resource_inputs {
        let has_match = dags.iter().any(|dag| {
            let entrypoints = crate::entrypoint::detect_entrypoints(dag);
            entrypoints
                .entrypoint_ports
                .iter()
                .any(|(_, port_name, _)| port_name.0 == ri.port_name)
        });
        if !has_match {
            let mut all_available: Vec<String> = Vec::new();
            for dag in dags {
                let entrypoints = crate::entrypoint::detect_entrypoints(dag);
                for (_, p, _) in &entrypoints.entrypoint_ports {
                    if !all_available.contains(&p.0) {
                        all_available.push(p.0.clone());
                    }
                }
            }
            panic!(
                "Pattern '{}': declared resource input '{}' (type '{}') has no matching \
                 entrypoint in any sub-DAG. Available entrypoints: {:?}",
                pattern_name, ri.port_name, ri.type_id, all_available
            );
        }
    }
}
