//! gunbc-dag: Repo-specific DAG configuration for gunbc.
//!
//! This crate contains the gunbc repo's specific configuration, including:
//! - CI pipeline definition
//! - Makefile generation
//! - Bootstrap tools
//! - Workspace DAG composition
//!
//! # Distinction from lib/tools/
//!
//! The crates in `lib/tools/` are general-purpose tool wrappers that could be
//! used by any project. This crate (`gunbc-dag`) contains configuration
//! specific to the gunbc repository itself.
//!
//! For example:
//! - `gunbc-clippy` (in lib/tools/) wraps the clippy CLI tool (general)
//! - `gunbc-dag::ci` defines gunbc's CI pipeline (repo-specific)

#![deny(dead_code)]
pub mod binaries;
pub mod bootstrap;
pub mod build;
pub mod ci;
pub mod codegen;
pub mod compiled_fns;
pub mod deps_tool;
#[allow(clippy::vec_init_then_push)] // Docgen uses vec-init-then-push patterns
pub mod docgen;
pub mod dry_run;
pub(crate) mod dsl_builder;
pub use dsl_builder::{
    build_aws_credential_graph_dsl, build_azure_credential_graph_dsl, build_clippy_graph_dsl,
    build_dimension_review_graph_dsl, build_gist_diff_graph_dsl, build_gist_recent_graph_dsl,
    build_gist_snapshot_graph_dsl, build_review_graph_dsl,
};
pub mod fs_env;
pub mod gist;
pub mod infra;

pub mod makegen;
pub mod mock_defaults;
pub mod policy;
pub mod pragma;
pub mod resolve;
pub mod resolve_service;
pub mod resources;
pub mod testgen_dag;
pub mod tool_runner;
pub mod workflow;
// Re-exports for convenience
pub use binaries::WorkspaceBinary;
pub use bootstrap::{bootstrap_signature, build_bootstrap_graph, BootstrapGraphOp, BootstrapOp};
pub use build::{build_build_graph, build_signature, BuildGraphOp, BuildOp};
pub use ci::{build_ci_graph, ci_signature, ci_workflow_config, CIGraphOp, CIOp};
pub use gunbc_lib_cloud_ops::env_requirements::{
    aws_github_actions_env_stub, azure_github_actions_env_stub, cloud_env_matrix,
    gcp_github_actions_env, gcp_local_env, gcp_metadata_env, CloudEnvRequirements,
    CLOUD_ENV_COMMON_OPTIONAL,
};
pub use codegen::{build_codegen_graph, codegen_signature, CodegenGraphOp, CodegenOp};
pub use docgen::{
    build_docgen_graph, DocgenGraphOp, DocgenOp, DocgenReadTarget, DOCGEN_READ_TARGETS,
};
pub use dry_run::wire_fs_env_write_mock;
pub use fs_env::{add_fs_env_root_node, wire_fs_env_write_edges};
pub use gunbc_ir::CODEGEN_STAMP_PATH;
pub use makegen::{
    build_makegen_graph, default_build_config, default_core_workflows, makegen_signature,
    render_github_actions_from_workflow_specs, render_gitignore,
    render_gitlab_ci_from_workflow_specs, render_justfile, render_makefile, workflow_specs_to_dag,
    BuildConfig, MakegenGraphOp, MakegenOp, WorkflowKind, WorkflowSpec,
};
pub use pragma::{build_pragma_graph, pragma_signature, PragmaGraphOp, PragmaOp};
pub use resolve::{resolve_lowered_dag, ResolveError};
pub use resources::{
    deps_config_resource_def, gitignore_resource_def, makefile_resource_def, testgen_resource_def,
};
pub use testgen_dag::{TestgenGraphOp, TestgenOp};
pub use tool_runner::{
    freshness_steps_planned, print_tool_header, run_tool,
    update_freshness_manifest_if_needed, RunToolOptions,
};
pub use workflow::{
    all_tool_workflow_names,
    bootstrap_workflow_spec, bootstrap_workflow_spec_with_registry, build_all_workflow_spec,
    build_all_workflow_spec_with_registry, check_slo, ci_unit_commands, ci_workflow_spec,
    ci_workflow_spec_with_registry, claim_handle_type_id, codegen_key, compilation_key,
    coordination_status, default_process_unit_registry,
    default_slo_budgets, deps_workflow_spec, deps_workflow_spec_with_registry,
    execute_workflow_plan, explain_plan, gist_diff_workflow_spec,
    gist_diff_workflow_spec_with_registry, gist_recent_workflow_spec,
    gist_recent_workflow_spec_with_registry, gist_snapshot_workflow_spec,
    gist_snapshot_workflow_spec_with_registry, gist_workflow_spec,
    gist_workflow_spec_with_registry, has_required_unit_contract,
    makegen_workflow_spec, makegen_workflow_spec_with_registry, plan_global_workflows,
    plan_workflow, plan_workflow_with_mode, pragma_workflow_spec,
    pragma_workflow_spec_with_registry, project_execute_set, prove_non_redundancy,
    render_execution_report, required_input_contract,
    required_output_contract,
    test_all_unit_commands,
    test_all_workflow_spec, test_all_workflow_spec_with_registry, tool_workflow_spec,
    top_slow_units, validate_conflicting_claims, validate_effectful_claim_declarations,
    validate_projection_equivalence, validate_required_claims, validate_workflow_admission,
    workflow_unit_commands, AggregateSpec, BlockedReason,
    CanonicalKeyPayload, CapabilityAction, CapabilityStatus, ClaimId, CodegenMissReason,
    CompilationMissReason, CompilationPhase, CoordinationStatus, DryRunMode, ExecuteProjection,
    ExecutionSummary, GlobalExecutionVertex, GlobalWorkflowPlan, InvariantViolation,
    MaterializationDigest, MaterializationKey, MissReason, NodePlan, PlanAction, PlanExplain,
    PlannerInputs, PlannerInputsByWorkflow, ProcessId, ProcessUnitRef, ProcessUnitRegistry,
    ProcessUnitSpec, ProjectionDrift, ReportSpec, SloBudget, SloResult,
    SlowUnit, UnitClaim, UnitCommand, UnitResult, WorkIdentity, WorkflowAdmissionError, WorkflowId,
    WorkflowNodeRef, WorkflowOp, WorkflowPlan,
    WorkflowPlannerError, WorkflowSpec as PlannerWorkflowSpec, WorkflowUnit, CODEGEN_ENSURE_UNIT,
    CODEGEN_PROCESS_ID, COMPILATION_ENSURE_UNIT, COMPILATION_PROCESS_ID, PORT_AFTER, PORT_COMMIT,
    PORT_RESULT, TYPE_WORKFLOW_RESULT,
};

// ============================================================================
// DagSpec Registry Helpers
// ============================================================================

/// Return DagSpec registrations originating from this crate.
pub fn dag_specs() -> Vec<&'static gunbc_testgen_registry::DagSpecDef> {
    gunbc_testgen_registry::iter_dag_specs()
        .filter(|spec| spec.origin_crate == env!("CARGO_CRATE_NAME"))
        .collect()
}

// ============================================================================
// Test linker hints
// ============================================================================
// Force the linker to include inventory submissions from dependency crates
// in the lib test binary. Without these, tool_target registrations from
// external crates are dead-stripped and derive_tool_defs() returns an
// incomplete set.
#[cfg(test)]
extern crate gunbc_clippy;
#[cfg(test)]
extern crate gunbc_lib_review;

// ============================================================================
// Cross-crate system model integration tests
// ============================================================================
// These tests require inventory symbols from gcp-ops, aws-ops, and transport
// to be linked. gunbc-dag depends on all three, so they run here.

#[cfg(test)]
mod system_model_integration {
    use gunbc_ir::system_model::{
        default_system_models, derive_contract_test_specs, generate_contract_test_harnesses,
        validate_store_behavior_mapping, Property, UpsertPhase,
    };

    #[test]
    fn contract_specs_follow_upsert_phase_rules() {
        let models = default_system_models();
        let specs = derive_contract_test_specs(&models);
        assert!(!specs.is_empty());
        assert!(specs.iter().any(|spec| spec.phase == UpsertPhase::Check
            && spec.required_all.contains(&Property::Deterministic)));
        assert!(specs.iter().any(|spec| {
            spec.phase == UpsertPhase::Create
                && spec.required_all.contains(&Property::WritesWorld)
                && spec
                    .required_any
                    .iter()
                    .any(|p| matches!(p, Property::Idempotent | Property::IdempotentWithKey))
        }));
    }

    #[test]
    fn contract_harnesses_render_type_safe_signatures() {
        let specs = derive_contract_test_specs(&default_system_models());
        let harnesses = generate_contract_test_harnesses(&specs);
        assert_eq!(harnesses.len(), specs.len());
        assert!(
            harnesses.iter().all(|h| h.starts_with("fn contract_")),
            "all harnesses should be generated contract fn signatures"
        );
        assert!(
            harnesses
                .iter()
                .any(|h| h.contains("gunbc_ir::transport::FileResponse")),
            "at least one harness should include concrete transport response type mappings"
        );
    }

    #[test]
    fn store_behavior_mapping_is_valid_for_gcs_and_s3() {
        validate_store_behavior_mapping(&default_system_models())
            .expect("store abstraction mapping should validate for both cloud providers");
    }
}
