//! gunbc-dag CI module.
//!
//! CI orchestration for the gunbc repo.

pub use gunbc_ir::transport::github_actions::WorkflowConfig;
pub use gunbc_primitives::EmbeddedFileExistsOp;

use gunbc_exec::DynOp;
use gunbc_ir::transport::github_actions::{
    checkout, gcp_workload_identity, rust_toolchain, ubuntu_latest, Integration, Permissions,
};
use gunbc_ir::CargoInvocation;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};
use gunbc_testgen_registry::iter_dag_specs;
use std::collections::BTreeSet;

/// Runtime op type for CI graphs.
pub type CIGraphOp = DynOp;

/// Get the declared signature for the ci workflow (auto-derived from DAG).
pub fn ci_signature() -> WorkflowSignature {
    match build_ci_graph() {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build ci DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
}

/// Get the integrations used by the CI workflow.
pub fn ci_integrations() -> Vec<Integration> {
    vec![checkout(), rust_toolchain(), gcp_workload_identity()]
}

/// Get the complete workflow configuration for CI.
pub fn ci_workflow_config() -> WorkflowConfig {
    let ci_cmd = CargoInvocation::composed("ci", "dag").command();
    WorkflowConfig::new("CI", ubuntu_latest(), ci_integrations())
        .with_run_command(format!("|\n          {ci_cmd} -- run"))
}

/// Get the required permissions for the CI workflow.
pub fn ci_workflow_permissions() -> Permissions {
    ci_workflow_config().permissions
}

/// Live-test secret env vars that must be exported in CI.
///
/// Derived from testgen target metadata (`live_required` and
/// `live_required_any_of`) to keep workflow secrets and test metadata in sync.
///
/// Note: `ACTIONS_ID_TOKEN_REQUEST_URL` and `ACTIONS_ID_TOKEN_REQUEST_TOKEN`
/// are automatically provided by GitHub Actions when `id-token: write` is
/// granted; they are excluded from repository-secret export lists.
pub fn ci_live_test_secrets() -> Vec<&'static str> {
    let mut secrets: BTreeSet<&'static str> = BTreeSet::new();

    for def in iter_dag_specs() {
        if let Some(required) = def.testgen.live_required {
            for &secret in required {
                if !is_github_actions_runtime_env(secret) {
                    secrets.insert(secret);
                }
            }
        }
        if let Some(any_of_groups) = def.testgen.live_required_any_of {
            for group in any_of_groups {
                for &secret in *group {
                    if !is_github_actions_runtime_env(secret) {
                        secrets.insert(secret);
                    }
                }
            }
        }
    }

    secrets.into_iter().collect()
}

/// GCP-specific subset of live-test secrets.
pub fn ci_gcp_secrets() -> Vec<&'static str> {
    ci_live_test_secrets()
        .into_iter()
        .filter(|name| name.starts_with("GCP_"))
        .collect()
}

fn is_github_actions_runtime_env(name: &str) -> bool {
    matches!(
        name,
        "ACTIONS_ID_TOKEN_REQUEST_URL" | "ACTIONS_ID_TOKEN_REQUEST_TOKEN"
    )
}

/// Build the CI graph from the DSL source.
pub fn build_ci_graph() -> Result<Dag<CIGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph("pipelines/ci.dag")
}
