mod context;
mod mocks;
mod render;
mod triplets;

pub use context::{
    build_context, build_context_with_default_roots, check_from_context, check_from_module_graph,
    compile_from_context, compile_from_context_with_options,
    compile_from_module_graph_with_options, compile_resolve_execute_from_context,
    execute_resolved_dag, resolve_configured_roots, resolve_default_roots,
};
pub use daglang_driver::{
    CheckOutput, CodegenLayer, CodegenTarget, CompileError, CompileOptions, CompileOutput,
};
pub use gunbc_exec::DynOp;
pub use gunbc_resolve::ResolveError;
pub use mocks::{
    makegen_check_mode_transport_mocks, makegen_dry_run_transport_mocks, makegen_entrypoint_mocks,
};
pub use render::{
    render_canonical_ir_json, render_expand, render_manifest, render_manifest_with_format,
    render_obligations, render_progress_with_format, render_topology_with_format,
};
pub use triplets::render_triplets;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[cfg(test)]
use triplets::collect_transport_triplets;

#[cfg(test)]
// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests;
