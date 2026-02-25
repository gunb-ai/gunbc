//! RuntimeBackend: production `Backend` impl for the compile+link pipeline (NF-4).
//!
//! Resolves extern symbols declared via `extern func` / `extern asset` in DSL
//! modules against the compiled fn registry and custom resolver surfaces. This
//! is the bridge between the linker's abstract `Backend` trait and the concrete
//! resolver dispatch in `resolve.rs`.
//!
//! The backend does **validation only** — it confirms symbols are resolvable
//! (returns `Some(ResolvedExternFunc/Asset)`) but does NOT produce `DynOp`
//! instances. Actual `DynOp` production stays in `resolve.rs`.

use gunbc_ir::linker::{Backend, ResolvedExternAsset, ResolvedExternFunc};
use gunbc_ir::symbol::ProgramSymbolId;

// ============================================================================
// Shared symbol registry
// ============================================================================

/// All extern func symbols resolvable by the runtime backend.
///
/// This list is the single source of truth — both `RuntimeBackend` and
/// `resolve_extern_call()` in `resolve.rs` consult it to avoid maintaining
/// parallel registries.
pub const KNOWN_EXTERN_FUNCS: &[(&str, &str)] = &[
    // std.markdown rendering fns
    ("std.markdown", "render_heading"),
    ("std.markdown", "render_code_block"),
    ("std.markdown", "render_bullet_list"),
    ("std.markdown", "render_numbered_list"),
    ("std.markdown", "render_tree"),
    ("std.markdown", "render_node"),
    ("std.markdown", "render_markdown"),
    // tools.gist compiled fns
    ("tools.gist", "build_snapshot_content"),
    ("tools.gist", "render_diff_markdown"),
    // tools.pragma compiled fns
    ("tools.pragma", "render_clippy_toml"),
    ("tools.pragma", "render_disallowed_methods_allowlist"),
    ("tools.pragma", "render_pragma_lint_policy"),
    // tools.bootstrap compiled fns
    ("tools.bootstrap", "prepare_scan_workspace"),
    ("tools.bootstrap", "parse_scan_result"),
    ("tools.bootstrap", "render_bootstrap_makefile"),
    ("tools.bootstrap", "render_bootstrap_gitignore"),
    // tools.makegen compiled fns
    ("tools.makegen", "load_registry"),
    ("tools.makegen", "render_makefile"),
    ("tools.makegen", "makegen"),
];

/// All extern asset symbols resolvable by the runtime backend.
pub const KNOWN_EXTERN_ASSETS: &[(&str, &str)] = &[
    // tools.makegen embedded asset
    ("tools.makegen", "makefile"),
];

/// Check if a (module, name) pair is a known extern func.
pub fn is_known_extern_func(module: &str, name: &str) -> bool {
    KNOWN_EXTERN_FUNCS
        .iter()
        .any(|(m, n)| *m == module && *n == name)
}

/// Check if a symbol is resolvable via std.resources dynamic dispatch.
///
/// Resource lifecycle funcs follow the pattern
/// `std.resources::resource_lifecycle::acquire::*` /
/// `std.resources::resource_lifecycle::release::*`. These don't appear in
/// the static registry because the resource name is extracted from the DSL
/// callable name at runtime.
///
/// Note: `ProgramSymbolId::name()` only returns the segment after the first
/// `::`, so we match on the full symbol string to handle nested `::` paths.
fn is_std_resources_symbol(sym: &ProgramSymbolId) -> bool {
    let s = sym.as_str();
    s.starts_with("std.resources::resource_lifecycle::acquire::")
        || s.starts_with("std.resources::resource_lifecycle::release::")
}

/// Check if a symbol is resolvable via tools.infra dispatch.
fn is_tools_infra_symbol(sym: &ProgramSymbolId) -> bool {
    sym.as_str() == "tools.infra::infra"
}

// ============================================================================
// RuntimeBackend
// ============================================================================

/// Production backend that resolves extern symbols against the compiled fn
/// registry, std.resources dynamic dispatch, and tools.infra dispatch.
pub struct RuntimeBackend;

impl Backend for RuntimeBackend {
    fn resolve_extern_func(&self, sym: &ProgramSymbolId) -> Option<ResolvedExternFunc> {
        let module = sym.module()?;
        let name = sym.name()?;

        // 1. Static compiled fn registry.
        if is_known_extern_func(module, name) {
            return Some(ResolvedExternFunc {
                symbol: sym.clone(),
                resolved_by: "compiled_fns".to_string(),
            });
        }

        // 2. std.resources dynamic dispatch (nested :: paths).
        if is_std_resources_symbol(sym) {
            return Some(ResolvedExternFunc {
                symbol: sym.clone(),
                resolved_by: "std.resources".to_string(),
            });
        }

        // 3. tools.infra dispatch.
        if is_tools_infra_symbol(sym) {
            return Some(ResolvedExternFunc {
                symbol: sym.clone(),
                resolved_by: "tools.infra".to_string(),
            });
        }

        None
    }

    fn resolve_extern_asset(&self, sym: &ProgramSymbolId) -> Option<ResolvedExternAsset> {
        let module = sym.module()?;
        let name = sym.name()?;

        if KNOWN_EXTERN_ASSETS
            .iter()
            .any(|(m, n)| *m == module && *n == name)
        {
            return Some(ResolvedExternAsset {
                symbol: sym.clone(),
                content_hash: format!("{module}::{name}::content"),
                resolved_by: "runtime_embed".to_string(),
            });
        }

        None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::linker::{link, Backend};
    use gunbc_ir::symbol::{OpRef, SymbolTable};

    #[test]
    fn runtime_backend_resolves_all_known_extern_funcs() {
        let backend = RuntimeBackend;
        for (module, name) in KNOWN_EXTERN_FUNCS {
            let sym = ProgramSymbolId::from_parts(module, name);
            assert!(
                backend.resolve_extern_func(&sym).is_some(),
                "RuntimeBackend should resolve extern func {module}::{name}"
            );
        }
    }

    #[test]
    fn runtime_backend_resolves_known_extern_assets() {
        let backend = RuntimeBackend;
        for (module, name) in KNOWN_EXTERN_ASSETS {
            let sym = ProgramSymbolId::from_parts(module, name);
            assert!(
                backend.resolve_extern_asset(&sym).is_some(),
                "RuntimeBackend should resolve extern asset {module}::{name}"
            );
        }
    }

    #[test]
    fn runtime_backend_resolves_resource_lifecycle() {
        let backend = RuntimeBackend;
        let sym =
            ProgramSymbolId::from_parts("std.resources", "resource_lifecycle::acquire::Filesystem");
        assert!(backend.resolve_extern_func(&sym).is_some());
        let sym =
            ProgramSymbolId::from_parts("std.resources", "resource_lifecycle::release::Filesystem");
        assert!(backend.resolve_extern_func(&sym).is_some());
    }

    #[test]
    fn runtime_backend_resolves_tools_infra() {
        let backend = RuntimeBackend;
        let sym = ProgramSymbolId::from_parts("tools.infra", "infra");
        assert!(backend.resolve_extern_func(&sym).is_some());
    }

    #[test]
    fn runtime_backend_rejects_unknown_symbol() {
        let backend = RuntimeBackend;
        let sym = ProgramSymbolId::from_parts("unknown.module", "unknown_func");
        assert!(backend.resolve_extern_func(&sym).is_none());
        assert!(backend.resolve_extern_asset(&sym).is_none());
    }

    #[test]
    fn link_succeeds_with_runtime_backend_for_known_symbols() {
        let backend = RuntimeBackend;
        let mut table = SymbolTable::new();
        let sym = ProgramSymbolId::from_parts("std.markdown", "render_heading");
        table.add_extern(sym.clone());
        table.add_op("node1".to_string(), OpRef::Extern(sym));

        let result = link(&table, &backend).expect("link should succeed for known symbol");
        assert_eq!(result.resolved_funcs.len(), 1);
    }

    #[test]
    fn known_extern_funcs_matches_compiled_fns_registry() {
        // Verify every KNOWN_EXTERN_FUNCS entry has a compiled fn implementation.
        for (module, name) in KNOWN_EXTERN_FUNCS {
            assert!(
                crate::compiled_fns::lookup_compiled_fn(module, name).is_some(),
                "KNOWN_EXTERN_FUNCS entry {module}::{name} has no compiled fn implementation"
            );
        }
    }
}
