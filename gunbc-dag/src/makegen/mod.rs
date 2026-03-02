//! gunbc-dag Makegen module.
//!
//! Makefile generation from gunbc DAG entrypoints.

pub mod gitignore;
pub mod justfile;
pub mod registry;
pub mod shared;

use daglang_emit::EmbeddedData;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};
use std::collections::HashMap;

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
pub fn makegen_signature() -> Result<WorkflowSignature, BuilderError> {
    build_makegen_graph().map(|dag| infer_signature(&dag))
}

/// Build makegen graph from the DSL source.
pub fn build_makegen_graph() -> Result<Dag<MakegenGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph_for_entrypoint("tools/makegen.dag", Some("makegen"))
}

/// Embedded asset key for precomputed makegen content.
pub const MAKEGEN_ASSET_KEY: &str = "tools.makegen::makefile";

/// Build embedded asset map for compile-time codegen.
pub fn build_embedded_data() -> Result<HashMap<String, EmbeddedData>, String> {
    let mut data = HashMap::new();
    data.insert(MAKEGEN_ASSET_KEY.to_string(), makegen_embedded_data()?);
    Ok(data)
}

/// Embedded makegen content payload.
pub fn makegen_embedded_data() -> Result<EmbeddedData, String> {
    Ok(EmbeddedData {
        module: "tools.makegen".to_string(),
        layer1_file_path: "src/embedded_makefile.txt".to_string(),
        layer2_ident: "makegen_content".to_string(),
        content: compute_makegen_content()?,
    })
}

/// Compute makegen content by rendering from discovered tools.
pub fn compute_makegen_content() -> Result<String, String> {
    let registry = ToolRegistry::default_registry()?;
    Ok(render_makefile(&registry))
}
