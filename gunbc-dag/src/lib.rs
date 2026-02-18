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
pub mod cloud_env;
pub mod codegen;
pub mod credential_lifecycle;
pub mod dag_viz;
#[allow(clippy::vec_init_then_push)] // Docgen uses vec-init-then-push patterns
pub mod docgen;
pub mod dry_run;
pub mod dsl_builder;
pub mod fs_env;
pub mod makegen;
pub mod mock_defaults;
pub mod policy;
pub mod pragma;
pub mod resolve;
pub mod resources;
pub mod testgen_dag;
pub mod tool_runner;
pub mod tool_testgen;
pub mod viewer;
pub mod workspace;

// Re-exports for convenience
pub use binaries::WorkspaceBinary;
pub use bootstrap::{bootstrap_signature, build_bootstrap_graph, BootstrapGraphOp, BootstrapOp};
pub use build::{build_build_graph, build_signature, BuildGraphOp, BuildOp};
pub use ci::{
    build_ci_graph, ci_signature, ci_workflow_config, CIGraphOp, CIOp,
};
pub use cloud_env::{
    aws_github_actions_env_stub, azure_github_actions_env_stub, cloud_env_matrix,
    gcp_github_actions_env, gcp_local_env, gcp_metadata_env, CloudEnvRequirements,
    CLOUD_ENV_COMMON_OPTIONAL,
};
pub use codegen::{build_codegen_graph, codegen_signature, CodegenGraphOp, CodegenOp};
pub use dag_viz::{build_dag_viz_graph, dag_viz_signature, DagVizGraphOp, DagVizMode};
pub use docgen::{
    build_docgen_graph, DocgenGraphOp, DocgenOp, DocgenReadTarget, DOCGEN_READ_TARGETS,
};
pub use dry_run::wire_fs_env_write_mock;
pub use dsl_builder::{
    build_bootstrap_graph_dsl, build_build_graph_dsl, build_ci_graph_dsl, build_codegen_graph_dsl,
    build_docgen_graph_dsl, build_makegen_graph_dsl, build_pragma_graph_dsl,
};
pub use fs_env::{add_fs_env_root_node, wire_fs_env_write_edges};
pub use gunbc_ir::CODEGEN_STAMP_PATH;
pub use makegen::{
    build_makegen_graph, default_build_config, makegen_signature, render_gitignore,
    render_makefile, BuildConfig, MakegenGraphOp, MakegenOp,
};
pub use pragma::{build_pragma_graph, pragma_signature, PragmaGraphOp, PragmaOp};
pub use resolve::{resolve_lowered_dag, ResolveError};
pub use resources::{
    deps_config_resource_def, gitignore_resource_def, makefile_resource_def, testgen_resource_def,
};
pub use testgen_dag::{TestgenGraphOp, TestgenOp};
pub use tool_runner::{print_tool_header, run_tool, RunToolOptions};
pub use workspace::{
    build_bootstrap_subdag, build_build_subdag, build_ci_subdag, build_clippy_lint_all_subdag,
    build_clippy_subdag, build_codegen_subdag, build_dag_viz_subdag, build_deps_generate_subdag,
    build_deps_install_subdag, build_docgen_subdag, build_gist_rust_subdag, build_gist_subdag,
    build_languages_subdag, build_makegen_subdag, build_pragma_subdag, build_testgen_subdag,
    build_workspace_dag, WorkspaceOp,
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
