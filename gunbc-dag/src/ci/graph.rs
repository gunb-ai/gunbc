//! DSL-backed graph builder for the CI tool.

use crate::dsl_builder::build_ci_graph_dsl;
use crate::WorkspaceBinary;
use gunbc_exec::DynOp;
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::github_actions::{
    checkout, gcp_workload_identity, rust_toolchain, ubuntu_latest, Integration, Permissions,
    WorkflowConfig,
};
use gunbc_ir::{BuilderError, Cardinality, Dag, WorkflowSignature};
use gunbc_testgen_registry::iter_dag_specs;
use std::collections::BTreeSet;

/// Runtime op type for CI graphs.
pub type CIGraphOp = DynOp;

/// Get the declared signature for the ci workflow.
pub fn ci_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        .with_output("deps_exists", "Bool", Cardinality::ONE)
        .with_output("deps_checked", "Bool", Cardinality::ONE)
        .with_output("deps_installed", "Int", Cardinality::ONE)
        .with_output("message", "String", Cardinality::ONE)
        .with_output("codegen_ran", "Bool", Cardinality::ONE)
        .with_output("prep_message", "String", Cardinality::ONE)
        .with_output("response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("skip", "Bool", Cardinality::ONE)
        .with_output("build_skipped", "Bool", Cardinality::ONE)
        .with_output("test_skipped", "Bool", Cardinality::ONE)
        .with_output("lint_skipped", "Bool", Cardinality::ONE)
        .with_output("skip_reason", "OptionalString", Cardinality::ZERO_OR_ONE)
        .with_output("overall_success", "Bool", Cardinality::ONE)
        .with_output("report", "String", Cardinality::ONE)
}

/// Get the integrations used by the CI workflow.
pub fn ci_integrations() -> Vec<Integration> {
    vec![checkout(), rust_toolchain(), gcp_workload_identity()]
}

/// Get the complete workflow configuration for CI.
pub fn ci_workflow_config() -> WorkflowConfig {
    let ci_cmd = WorkspaceBinary::Ci.command();
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
    build_ci_graph_dsl()
}

/// Build the CI graph with a compatibility mode parameter.
///
/// Mode-specific behavior is controlled by runtime `check_mode` inputs.
pub fn build_ci_graph_with_mode(mode: ExecMode) -> Result<Dag<CIGraphOp>, BuilderError> {
    let _ = mode;
    build_ci_graph()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ci_graph_from_dsl() {
        let dag = build_ci_graph().expect("ci DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn ci_secrets_exclude_github_runtime_tokens() {
        let secrets = ci_live_test_secrets();
        assert!(!secrets.contains(&"ACTIONS_ID_TOKEN_REQUEST_URL"));
        assert!(!secrets.contains(&"ACTIONS_ID_TOKEN_REQUEST_TOKEN"));
    }
}
