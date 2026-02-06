//! Auto-discovered tool target registry.
//!
//! This crate provides:
//! - A registry for tool targets (via `inventory`)
//! - Metadata for CLI generation, Makefile generation, and DAG composition
//!
//! # Design
//!
//! Mirrors the `gunbc-testgen-registry` pattern: tools register themselves
//! using `#[tool_target]` (from `gunbc-tool-registry-macros`), and downstream
//! consumers discover them via `iter_tool_targets()`.
//!
//! This crate is a leaf crate with no dependency on `gunbc-codegen` or
//! `gunbc-ir`. All fields use `&'static str` so registration can be const.
//! Conversion to richer types (e.g., `ToolDef`) happens in consuming code.

#![deny(dead_code)]
// Re-export inventory so macros can submit without depending on it directly.
pub use inventory;

/// A registered tool target (auto-discovered via inventory).
///
/// Uses `&'static str` fields so registration can be const.
/// This struct carries enough metadata for:
/// - CLI binary generation (graph builder, entrypoints)
/// - Makefile target generation (invocation, tool name)
/// - DAG composition (builder call expression)
#[derive(Debug)]
pub struct ToolRegistration {
    /// Originating crate name (from `CARGO_CRATE_NAME`, for path rewriting)
    pub origin_crate: &'static str,
    /// Cargo crate name (e.g., "gunbc-gist")
    pub crate_name: &'static str,
    /// Tool name for CLI and display (e.g., "gist")
    pub tool_name: &'static str,
    /// Short description
    pub description: &'static str,
    /// Graph builder call expression (e.g., "build_gist_graph(GistMode::Snapshot, ext, pub)")
    ///
    /// This replaces `GraphBuilderId` — the expression is validated at macro
    /// expansion time (same mechanism as testgen's `builder` field).
    pub graph_builder_call: &'static str,
    /// Arguments to pass to graph builder (e.g., "extensions.clone(), public")
    pub graph_builder_args: &'static str,
    /// Whether the graph builder returns `Result<Dag, BuilderError>`
    pub returns_result: bool,
    /// Output port to check for success (e.g., "overall_success").
    /// If this port is false, the generated CLI exits with code 1.
    pub success_port: Option<&'static str>,
    /// Enable step mode — generates `step <node>` subcommand for CI providers.
    pub enable_step_mode: bool,
    /// Custom import line (if different from default crate:: pattern)
    pub custom_import: Option<&'static str>,
}

impl ToolRegistration {
    /// Convert module paths from the origin crate form to `crate::` paths.
    ///
    /// `module_path!()` uses the crate identifier form (hyphens → underscores).
    /// This strips the origin crate prefix and rewrites as `crate::`.
    pub fn to_crate_path(&self, path: &str) -> String {
        let origin_ident = self.origin_crate.replace('-', "_");
        let prefix = format!("{}::", origin_ident);
        if let Some(stripped) = path.strip_prefix(&prefix) {
            format!("crate::{}", stripped)
        } else {
            path.to_string()
        }
    }
}

inventory::collect!(ToolRegistration);

/// Iterate over all registered tool targets.
pub fn iter_tool_targets() -> impl Iterator<Item = &'static ToolRegistration> {
    inventory::iter::<ToolRegistration>.into_iter()
}
