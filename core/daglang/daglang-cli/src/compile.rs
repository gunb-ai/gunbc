mod context;
mod render;
mod triplets;

pub use context::{
    build_context, check_from_context, compile_from_context, compile_from_context_with_options,
    compile_resolve_execute_from_context, execute_resolved_dag,
};
pub use daglang_driver::{CheckOutput, CompileError, CompileOptions, CompileOutput};
pub use daglang_exec_bridge::{
    makegen_check_mode_transport_mocks, makegen_dry_run_transport_mocks, makegen_entrypoint_mocks,
    resolve_lowered_dag, ResolveDagError, ResolvedOp,
};
pub use render::{render_expand, render_manifest, render_manifest_with_format, render_obligations};
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
