//! gunbc-dag Makegen module.
//!
//! Makefile generation from gunbc DAG entrypoints.

pub mod ci_render;
pub mod gitignore;
pub mod graph;
pub mod justfile;
pub mod ops;
pub mod registry;
pub mod render;

pub mod graph_mock;

pub use ci_render::{
    render_github_actions_from_workflow_specs, render_gitlab_ci_from_workflow_specs,
    workflow_specs_to_dag,
};
pub use gitignore::{derive_categories, render_gitignore, GitignoreRenderer};
pub use graph::{build_makegen_graph, makegen_signature, MakegenGraphOp};
pub use justfile::{render_justfile, render_justfile_with_config, JustfileRenderer};
pub use ops::MakegenOp;
pub use registry::{
    default_build_config, default_core_workflows, default_meta_targets, BuildConfig, BuildSystem,
    ConfigField, EntrypointParam, FixAlias, MetaTarget, ResourceNeed, ResourceTargetMap, ToolInfo,
    ToolRegistry, WorkflowKind, WorkflowSpec,
};
pub use render::{render_makefile, render_makefile_with_config};

// ============================================================================
// Tool Target Registrations
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "makegen",
    crate_name = "gunbc-makegen",
    description = "Generate Makefile from tool registry",
    builder = "build_makegen_graph",
    import = "use gunbc_makegen::build_makegen_graph;",
    mock_spec = "gunbc_dag::makegen::graph_mock::makegen_mock_spec()",
    package = "dag",
    binary = "makegen",
    entrypoints = r#"[{"port_name":"path","type_id":"String","short":"o","default":"Makefile","help":"Output Makefile path","make_var":"OUTPUT"}]"#,
    dsl_module = "makegen",
    has_invocation,
    returns_result
)]
pub fn makegen_tool() {}

#[cfg(test)]
mod generated_tests {
    include!("generated_tests.rs");
}
