//! Canonical program symbol identity.
//!
//! `ProgramSymbolId` is the stable key for extern symbols declared via
//! `extern func` / `extern asset` in DSL modules. Used by `resolve_extern_call()`
//! in the compile-time resolver.

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// ProgramSymbolId — canonical program symbol identity
// ============================================================================

/// A canonical program symbol identifier.
///
/// Derived from the DSL module path + item name. Two symbols with the
/// same `ProgramSymbolId` are the same program entity. The string form is the
/// stable key for hashing, ordering, and deterministic diagnostics.
///
/// Examples:
/// - `"tools.makegen::render_makefile"` (func)
/// - `"pipelines.ci::ci"` (pipeline)
/// - `"tools.pragma::clippy_toml_content"` (extern asset)
///
/// Note: Named `ProgramSymbolId` (not `SymbolId`) to avoid collision with
/// the visual symbol system in `symbols.rs` / `generated/mod.rs`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProgramSymbolId(pub String);

impl ProgramSymbolId {
    /// Create a new symbol ID from a canonical path.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Create a symbol ID from module + name components.
    pub fn from_parts(module: &str, name: &str) -> Self {
        Self(format!("{module}::{name}"))
    }

    /// The canonical string form.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The module portion (before `::`) if present.
    pub fn module(&self) -> Option<&str> {
        self.0.split("::").next()
    }

    /// The name portion (after `::`) if present.
    pub fn name(&self) -> Option<&str> {
        self.0.split("::").nth(1)
    }
}

impl fmt::Display for ProgramSymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ProgramSymbolId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ProgramSymbolId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_id_from_parts() {
        let sym = ProgramSymbolId::from_parts("tools.makegen", "render_makefile");
        assert_eq!(sym.as_str(), "tools.makegen::render_makefile");
        assert_eq!(sym.module(), Some("tools.makegen"));
        assert_eq!(sym.name(), Some("render_makefile"));
    }

    #[test]
    fn symbol_id_ordering_is_deterministic() {
        let a = ProgramSymbolId::new("a::b");
        let b = ProgramSymbolId::new("a::c");
        let c = ProgramSymbolId::new("b::a");
        let mut syms = vec![c.clone(), a.clone(), b.clone()];
        syms.sort();
        assert_eq!(syms, vec![a, b, c]);
    }
}
