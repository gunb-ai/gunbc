//! gunbc-dag Makegen module.
//!
//! Makefile generation from gunbc DAG entrypoints.

pub mod ci_render;
pub mod gitignore;
pub mod justfile;
pub mod registry;
pub mod shared;

use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

pub use ci_render::{
    render_github_actions_from_workflow_specs, render_gitlab_ci_from_workflow_specs,
    workflow_specs_to_dag,
};
pub use gitignore::{derive_categories, render_gitignore, GitignoreRenderer};
pub use justfile::{render_justfile, render_justfile_with_config, JustfileRenderer};
pub use registry::{
    default_build_config, default_core_workflows, default_meta_targets, BuildConfig, BuildSystem,
    ConfigField, EntrypointParam, FixAlias, MetaTarget, ResourceNeed, ResourceTargetMap, ToolInfo,
    ToolRegistry, WorkflowKind, WorkflowSpec,
};
pub use shared::render_makefile;

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
    crate::dsl_builder::build_dsl_graph_for_entrypoint("tools/makegen.dag", Some("makegen"))
}
