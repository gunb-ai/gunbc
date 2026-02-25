//! Linker: resolves extern symbols through backend interfaces (NF-3).
//!
//! The link step sits between lowering and emission:
//!
//! ```text
//! .dag → parse → typecheck → lower → [LINK] → emit
//! ```
//!
//! It takes a lowered DAG + symbol table and resolves all extern symbols
//! through a `Backend` trait. Missing symbols are hard errors — no fallbacks.

use crate::symbol::{OpRef, ProgramSymbolId, SymbolTable};
use std::fmt;

// ============================================================================
// Backend trait — extern symbol resolution
// ============================================================================

/// A resolved extern function implementation.
#[derive(Debug, Clone)]
pub struct ResolvedExternFunc {
    /// The symbol this resolves.
    pub symbol: ProgramSymbolId,
    /// Human-readable description of the resolution source.
    pub resolved_by: String,
}

/// A resolved extern asset.
#[derive(Debug, Clone)]
pub struct ResolvedExternAsset {
    /// The symbol this resolves.
    pub symbol: ProgramSymbolId,
    /// Content hash of the resolved asset bytes.
    pub content_hash: String,
    /// Human-readable description of the resolution source.
    pub resolved_by: String,
}

/// Backend interface for extern symbol resolution.
///
/// Implementations provide resolution for extern funcs and assets.
/// The linker calls these methods for each extern symbol in the
/// reachable graph. Missing resolutions are hard link errors.
pub trait Backend {
    /// Resolve an extern func symbol to its implementation.
    fn resolve_extern_func(&self, sym: &ProgramSymbolId) -> Option<ResolvedExternFunc>;

    /// Resolve an extern asset symbol to its content.
    fn resolve_extern_asset(&self, sym: &ProgramSymbolId) -> Option<ResolvedExternAsset>;
}

// ============================================================================
// Link errors
// ============================================================================

/// Errors from the link step.
#[derive(Debug, Clone)]
pub enum LinkError {
    /// An extern func symbol could not be resolved by any backend.
    MissingExternFunc {
        symbol: ProgramSymbolId,
        required_by_node: String,
    },
    /// An extern asset symbol could not be resolved by any backend.
    MissingExternAsset {
        symbol: ProgramSymbolId,
        required_by_node: String,
    },
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingExternFunc {
                symbol,
                required_by_node,
            } => write!(
                f,
                "missing extern func `{symbol}` (required by node `{required_by_node}`)"
            ),
            Self::MissingExternAsset {
                symbol,
                required_by_node,
            } => write!(
                f,
                "missing extern asset `{symbol}` (required by node `{required_by_node}`)"
            ),
        }
    }
}

impl std::error::Error for LinkError {}

// ============================================================================
// Link result
// ============================================================================

/// Result of a successful link step.
#[derive(Debug, Clone, Default)]
pub struct LinkResult {
    /// Successfully resolved extern funcs.
    pub resolved_funcs: Vec<ResolvedExternFunc>,
    /// Successfully resolved extern assets.
    pub resolved_assets: Vec<ResolvedExternAsset>,
    /// Diagnostics (informational, not errors).
    pub diagnostics: Vec<String>,
}

// ============================================================================
// Linker
// ============================================================================

/// Link a symbol table against a backend, resolving all extern symbols.
///
/// Returns `Ok(LinkResult)` if all externs resolve, or `Err(Vec<LinkError>)`
/// with deterministic diagnostic ordering (sorted by symbol name) if any
/// are missing.
pub fn link(table: &SymbolTable, backend: &dyn Backend) -> Result<LinkResult, Vec<LinkError>> {
    let mut result = LinkResult::default();
    let mut errors = Vec::new();

    // Collect extern symbols and their requiring nodes.
    let mut extern_requirements: Vec<(&ProgramSymbolId, &str)> = Vec::new();
    for (node_id, op_ref) in &table.ops {
        if let OpRef::Extern(sym) = op_ref {
            extern_requirements.push((sym, node_id.as_str()));
        }
    }

    // Deterministic ordering: sort by symbol name.
    extern_requirements.sort_by_key(|(sym, _)| sym.as_str().to_string());

    for (sym, node_id) in extern_requirements {
        // Try func resolution first, then asset.
        if let Some(resolved) = backend.resolve_extern_func(sym) {
            result.resolved_funcs.push(resolved);
        } else if let Some(resolved) = backend.resolve_extern_asset(sym) {
            result.resolved_assets.push(resolved);
        } else {
            errors.push(LinkError::MissingExternFunc {
                symbol: sym.clone(),
                required_by_node: node_id.to_string(),
            });
        }
    }

    if errors.is_empty() {
        Ok(result)
    } else {
        Err(errors)
    }
}

/// A no-op backend that resolves nothing. Used for testing link errors.
pub struct EmptyBackend;

impl Backend for EmptyBackend {
    fn resolve_extern_func(&self, _sym: &ProgramSymbolId) -> Option<ResolvedExternFunc> {
        None
    }

    fn resolve_extern_asset(&self, _sym: &ProgramSymbolId) -> Option<ResolvedExternAsset> {
        None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBackend {
        funcs: Vec<String>,
    }

    impl Backend for MockBackend {
        fn resolve_extern_func(&self, sym: &ProgramSymbolId) -> Option<ResolvedExternFunc> {
            if self.funcs.contains(&sym.0) {
                Some(ResolvedExternFunc {
                    symbol: sym.clone(),
                    resolved_by: "mock".to_string(),
                })
            } else {
                None
            }
        }

        fn resolve_extern_asset(&self, _sym: &ProgramSymbolId) -> Option<ResolvedExternAsset> {
            None
        }
    }

    #[test]
    fn link_succeeds_when_all_externs_resolved() {
        let mut table = SymbolTable::new();
        table.add_extern(ProgramSymbolId::new("runtime::handler"));
        table.add_op(
            "node1".to_string(),
            OpRef::Extern(ProgramSymbolId::new("runtime::handler")),
        );

        let backend = MockBackend {
            funcs: vec!["runtime::handler".to_string()],
        };

        let result = link(&table, &backend).expect("link should succeed");
        assert_eq!(result.resolved_funcs.len(), 1);
        assert_eq!(
            result.resolved_funcs[0].symbol.as_str(),
            "runtime::handler"
        );
    }

    #[test]
    fn link_fails_with_hard_error_for_missing_extern() {
        let mut table = SymbolTable::new();
        table.add_extern(ProgramSymbolId::new("missing::func"));
        table.add_op(
            "node1".to_string(),
            OpRef::Extern(ProgramSymbolId::new("missing::func")),
        );

        let errors = link(&table, &EmptyBackend).expect_err("link should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0].to_string().contains("missing::func"),
            "error should name the missing symbol: {}",
            errors[0]
        );
    }

    #[test]
    fn link_errors_are_deterministically_ordered() {
        let mut table = SymbolTable::new();
        // Add in non-sorted order.
        table.add_op(
            "node_z".to_string(),
            OpRef::Extern(ProgramSymbolId::new("z::func")),
        );
        table.add_op(
            "node_a".to_string(),
            OpRef::Extern(ProgramSymbolId::new("a::func")),
        );
        table.add_op(
            "node_m".to_string(),
            OpRef::Extern(ProgramSymbolId::new("m::func")),
        );

        let errors = link(&table, &EmptyBackend).expect_err("link should fail");
        assert_eq!(errors.len(), 3);

        // Errors should be sorted by symbol name.
        let symbols: Vec<&str> = errors
            .iter()
            .map(|e| match e {
                LinkError::MissingExternFunc { symbol, .. } => symbol.as_str(),
                LinkError::MissingExternAsset { symbol, .. } => symbol.as_str(),
            })
            .collect();
        assert_eq!(symbols, vec!["a::func", "m::func", "z::func"]);
    }

    #[test]
    fn link_with_no_externs_succeeds_immediately() {
        let table = SymbolTable::new();
        let result = link(&table, &EmptyBackend).expect("link should succeed with no externs");
        assert!(result.resolved_funcs.is_empty());
        assert!(result.resolved_assets.is_empty());
    }
}
