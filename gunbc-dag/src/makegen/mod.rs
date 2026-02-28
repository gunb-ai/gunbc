//! gunbc-dag Makegen module.
//!
//! Makefile generation from gunbc DAG entrypoints.

pub mod gitignore;
pub mod justfile;
pub mod registry;
pub mod shared;

use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

pub use gitignore::{derive_categories, render_gitignore, GitignoreRenderer};
pub use justfile::{render_justfile, render_justfile_with_config, JustfileRenderer};
pub use registry::{
    default_build_config, BuildCommand, BuildConfig, BuildSystem, EntrypointParam, ExtraTarget,
    ToolInfo, ToolRegistry, WorkflowKind, WorkflowSpec,
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
