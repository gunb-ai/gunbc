//! gunbc-dag Makegen module.
//!
//! Makefile generation from gunbc DAG entrypoints.

pub mod ci_render;
pub mod gitignore;
pub mod justfile;
pub mod ops;
pub mod registry;
pub mod render;

use crate::dsl_builder::build_makegen_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

pub use ci_render::{
    render_github_actions_from_workflow_specs, render_gitlab_ci_from_workflow_specs,
    workflow_specs_to_dag,
};
pub use gitignore::{derive_categories, render_gitignore, GitignoreRenderer};
pub use justfile::{render_justfile, render_justfile_with_config, JustfileRenderer};
pub use ops::MakegenOp;
pub use registry::{
    default_build_config, default_core_workflows, default_meta_targets, BuildConfig, BuildSystem,
    ConfigField, EntrypointParam, FixAlias, MetaTarget, ResourceNeed, ResourceTargetMap, ToolInfo,
    ToolRegistry, WorkflowKind, WorkflowSpec,
};
pub use render::{render_makefile, render_makefile_with_config};

/// Runtime op type for makegen graphs.
pub type MakegenGraphOp = DynOp;

/// Get the declared signature for the makegen workflow (auto-derived from DAG).
pub fn makegen_signature() -> WorkflowSignature {
    match build_makegen_graph() {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build makegen DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
}

/// Build makegen graph from the DSL source.
pub fn build_makegen_graph() -> Result<Dag<MakegenGraphOp>, BuilderError> {
    build_makegen_graph_dsl()
}

// ============================================================================
// Tool Target Registrations
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "makegen",
    crate_name = "gunbc-makegen",
    description = "Generate Makefile from tool registry",
    builder = "build_makegen_graph",
    import = "use gunbc_makegen::build_makegen_graph;",
    package = "dag",
    binary = "makegen",
    entrypoints = r#"[{"port_name":"path","type_id":"String","short":"o","default":"Makefile","help":"Output Makefile path","make_var":"OUTPUT"}]"#,
    dsl_module = "makegen",
    outputs = "Makefile",
    provides = "Makefile",
    has_invocation,
    returns_result
)]
pub fn makegen_tool() {}
