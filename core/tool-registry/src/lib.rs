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
    /// Fully-qualified Rust expression for the MockSpec function.
    /// Used by generated CLIs for `--dry-run` boundary mocking.
    pub mock_spec_call: Option<&'static str>,
    /// JSON-encoded array of entrypoint definitions.
    ///
    /// Each entry: `{"port_name":"...","type_id":"...","cardinality":"ONE",
    ///   "short":null,"default":null,"help":"...","make_var":null}`
    ///
    /// Empty string means no entrypoints.
    pub entrypoints_json: &'static str,
    /// Cargo package name (e.g., "gist"). Used for invocation and Makefile generation.
    /// When None, the tool has no runnable binary.
    pub package: Option<&'static str>,
    /// Binary name (e.g., "gist-diff"). When None, defaults to tool_name.
    pub binary: Option<&'static str>,
    /// Whether this tool has a runnable binary (generates a CargoInvocation).
    /// When false, the tool is library-only or a sub-DAG component.
    pub has_invocation: bool,
    /// DSL module name (file stem in `dsl/tools/` or `dsl/pipelines/`).
    ///
    /// When set, this tool is derived from the named `.dag` file.
    /// Multiple tools can share a DSL module (e.g., "gist" → gist, gist-diff, gist-recent).
    /// Used by codegen and makegen to validate DSL coverage without hardcoded lists.
    pub dsl_module: Option<&'static str>,
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

/// Collect DSL module → tool target name mappings from the registry.
///
/// Returns a map from DSL module name to the set of tool target names
/// derived from that module. Only includes tools with `dsl_module` set.
pub fn dsl_module_to_targets() -> std::collections::BTreeMap<&'static str, Vec<&'static str>> {
    let mut map: std::collections::BTreeMap<&'static str, Vec<&'static str>> =
        std::collections::BTreeMap::new();
    for reg in iter_tool_targets() {
        if let Some(module) = reg.dsl_module {
            map.entry(module).or_default().push(reg.tool_name);
        }
    }
    map
}
