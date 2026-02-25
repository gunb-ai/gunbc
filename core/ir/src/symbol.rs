//! Minimal symbol model for the compile+link pipeline (NF-2).
//!
//! Two ID types:
//! - `SymbolId`: Named program symbols (funcs, pipelines, assets), derived
//!   from canonical symbol paths (e.g., `"tools.makegen::render_makefile"`).
//! - `OpRef`: Classifies each operation as Intrinsic (compiler-primitive),
//!   Call (DSL-defined symbol), or Extern (must link via backend).
//!
//! These abstractions sit above `LoweredOp` — the lowerer produces
//! `LoweredOp` nodes, and the symbol model classifies them for the
//! link step (NF-3).

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// SymbolId — canonical program symbol identity
// ============================================================================

/// A canonical program symbol identifier.
///
/// Derived from the DSL module path + item name. Two symbols with the
/// same `SymbolId` are the same program entity. The string form is the
/// stable key for hashing, ordering, and deterministic diagnostics.
///
/// Examples:
/// - `"tools.makegen::render_makefile"` (func)
/// - `"pipelines.ci::ci"` (pipeline)
/// - `"tools.pragma::clippy_toml_content"` (extern asset)
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
// IntrinsicOp — compiler-primitive operations
// ============================================================================

/// Compiler-primitive operations that don't correspond to named symbols.
///
/// These are structural operations injected by the lowerer for DAG wiring,
/// pattern expansion, and transport phases. They are always available and
/// never need linking.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntrinsicOp {
    /// File system environment provider.
    FsEnv,
    /// Parameter source (wires a callable's input parameter).
    ParamSource { callable: String, param: String },
    /// Literal constant source.
    LiteralSource,
    /// File read preparation.
    PrepareFileRead,
    /// File read execution.
    ExecuteFileRead,
    /// Content equality comparison.
    CompareEquality,
    /// File write preparation.
    PrepareFileWrite,
    /// File write execution.
    ExecuteFileWrite,
    /// Loop unpack (element extraction from collection).
    LoopUnpack,
    /// Loop pack (element collection back to list).
    LoopPack,
    /// Branch merge (convergence after conditional).
    BranchMerge,
    /// Collection operation (map, filter, fold, etc.).
    Collection { kind: String },
    /// Service transport prepare phase.
    ServiceTransportPrepare,
    /// Service transport execute phase.
    ServiceTransportExecute,
    /// Service transport parse phase.
    ServiceTransportParse,
    /// Resource lifecycle acquire.
    ResourceAcquire { resource: String },
    /// Resource lifecycle release.
    ResourceRelease { resource: String },
}

// ============================================================================
// OpRef — operation classification
// ============================================================================

/// Classifies each DAG operation for the link step.
///
/// The lowerer produces `LoweredOp` nodes. `OpRef::classify()` maps each
/// to one of three forms:
///
/// - `Intrinsic`: Compiler-primitive (always available, never linked).
/// - `Call`: DSL-defined symbol (resolved at compile time).
/// - `Extern`: Must be resolved by a backend at link time (NF-3).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpRef {
    /// Compiler-primitive operation (transport phases, patterns, etc.).
    Intrinsic(IntrinsicOp),
    /// Call to a DSL-defined symbol (func, pipeline).
    Call(ProgramSymbolId),
    /// Reference to an extern symbol that must be linked.
    Extern(ProgramSymbolId),
    /// Operation that could not be classified (error path).
    Unresolved { detail: String },
}

impl OpRef {
    /// Returns true if this is an extern reference that needs linking.
    pub fn needs_linking(&self) -> bool {
        matches!(self, Self::Extern(_))
    }

    /// Returns the symbol ID if this is a Call or Extern.
    pub fn symbol(&self) -> Option<&ProgramSymbolId> {
        match self {
            Self::Call(sym) | Self::Extern(sym) => Some(sym),
            _ => None,
        }
    }
}

impl fmt::Display for OpRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Intrinsic(op) => write!(f, "intrinsic:{op:?}"),
            Self::Call(sym) => write!(f, "call:{sym}"),
            Self::Extern(sym) => write!(f, "extern:{sym}"),
            Self::Unresolved { detail } => write!(f, "unresolved:{detail}"),
        }
    }
}

// ============================================================================
// SymbolTable — collected symbols from a DAG
// ============================================================================

/// A table of all symbols referenced by a DAG.
///
/// Built during lowering, consumed by the link step (NF-3) to verify
/// all extern symbols are resolved by the backend.
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// All DSL-defined symbols (funcs, pipelines).
    pub defined: Vec<ProgramSymbolId>,
    /// All extern symbols that must be linked.
    pub externs: Vec<ProgramSymbolId>,
    /// All operation references (for diagnostics).
    pub ops: Vec<(String, OpRef)>,
}

impl SymbolTable {
    /// Create an empty symbol table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a defined symbol.
    pub fn add_defined(&mut self, sym: ProgramSymbolId) {
        self.defined.push(sym);
    }

    /// Record an extern symbol requirement.
    pub fn add_extern(&mut self, sym: ProgramSymbolId) {
        self.externs.push(sym);
    }

    /// Record an operation reference.
    pub fn add_op(&mut self, node_id: String, op_ref: OpRef) {
        self.ops.push((node_id, op_ref));
    }

    /// All extern symbols that need linking.
    pub fn unresolved_externs(&self) -> Vec<&ProgramSymbolId> {
        self.externs
            .iter()
            .filter(|ext| !self.defined.iter().any(|def| def == *ext))
            .collect()
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

    #[test]
    fn op_ref_needs_linking() {
        assert!(OpRef::Extern(ProgramSymbolId::new("ext::func")).needs_linking());
        assert!(!OpRef::Call(ProgramSymbolId::new("local::func")).needs_linking());
        assert!(!OpRef::Intrinsic(IntrinsicOp::FsEnv).needs_linking());
    }

    #[test]
    fn symbol_table_tracks_unresolved_externs() {
        let mut table = SymbolTable::new();
        table.add_defined(ProgramSymbolId::new("local::a"));
        table.add_extern(ProgramSymbolId::new("ext::b"));
        table.add_extern(ProgramSymbolId::new("local::a")); // defined, not unresolved

        let unresolved = table.unresolved_externs();
        assert_eq!(unresolved.len(), 1);
        assert_eq!(unresolved[0].as_str(), "ext::b");
    }
}
