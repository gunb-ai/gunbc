//! WorkspaceOp: Unified operation enum for fractal DAG composition.
//!
//! This enum wraps all domain operations, primitives, and transport ops
//! into a single type that can be used throughout the workspace DAG.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use std::collections::HashMap;

// Domain ops - local (repo-specific)
use crate::bootstrap::BootstrapOp;
use crate::ci::{CIOp, EnvOp};
use crate::makegen::MakegenOp;

// Domain ops - external (general tools)
use gunbc_buck2::Buck2Op;
use gunbc_clippy::CliToolOp;
use gunbc_deps::DepsOp;
use gunbc_gist::GistOps;
use gunbc_ir::LanguageOp;

// Infrastructure ops
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrimitiveOp;

/// Unified operation enum for the workspace DAG.
///
/// All tool, language, primitive, and transport operations are wrapped
/// in this single enum, enabling fractal composition of SubDags.
///
/// # Categories
///
/// - **Domain ops**: Tool-specific pure operations (Ci, Deps, Makegen, etc.)
/// - **Language ops**: Language/format characteristics (from Languages DAG)
/// - **Primitive ops**: Reusable pure operations (parsing, file prep, etc.)
/// - **Transport ops**: I/O boundary operations
#[derive(Debug, Clone)]
pub enum WorkspaceOp {
    // ========================================================================
    // Domain Ops (tool-specific pure operations)
    // ========================================================================
    /// CI workflow operations
    Ci(CIOp),
    /// Dependency management operations
    Deps(DepsOp),
    /// Makefile generation operations
    Makegen(MakegenOp),
    /// Gist operations
    Gist(GistOps),
    /// Bootstrap operations
    Bootstrap(BootstrapOp),
    /// Buck2 build operations
    Buck2(Buck2Op),
    /// Clippy/CLI tool operations
    Clippy(CliToolOp),
    /// Environment node that provides tools (I/O boundary for tool acquisition)
    Env(EnvOp),

    // ========================================================================
    // Language Ops (from Languages DAG)
    // ========================================================================
    /// Language and format characteristic operations
    Language(LanguageOp),

    // ========================================================================
    // Infrastructure Ops
    // ========================================================================
    /// Reusable primitive operations (parsing, collections, etc.)
    Primitive(PrimitiveOp),
    /// Transport boundary operations (actual I/O)
    Transport(TransportOps),
}

impl Default for WorkspaceOp {
    fn default() -> Self {
        // Default to transport execute - safe no-op when properly guarded
        WorkspaceOp::Transport(TransportOps::Execute)
    }
}

impl Executable for WorkspaceOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            // Domain ops
            WorkspaceOp::Ci(op) => op.execute(inputs),
            WorkspaceOp::Deps(op) => op.execute(inputs),
            WorkspaceOp::Makegen(op) => op.execute(inputs),
            WorkspaceOp::Gist(op) => op.execute(inputs),
            WorkspaceOp::Bootstrap(op) => op.execute(inputs),
            WorkspaceOp::Buck2(op) => op.execute(inputs),
            // CliToolOp has its own execute signature - wrap it
            WorkspaceOp::Clippy(op) => op
                .execute()
                .map_err(|e| ExecError::new(format!("CliToolOp error: {}", e))),
            // Env node does tool acquisition
            WorkspaceOp::Env(op) => op.execute(inputs),
            // Language ops
            WorkspaceOp::Language(_op) => {
                // LanguageOp nodes are mostly config nodes - return empty for now
                // In the future, this could dispatch to language-specific execution
                Ok(HashMap::new())
            }
            // Infrastructure ops
            WorkspaceOp::Primitive(op) => op.execute(inputs),
            WorkspaceOp::Transport(op) => op.execute(inputs),
        }
    }
}

// ============================================================================
// Conversion traits for ergonomic SubDag construction
// ============================================================================

impl From<CIOp> for WorkspaceOp {
    fn from(op: CIOp) -> Self {
        WorkspaceOp::Ci(op)
    }
}

impl From<DepsOp> for WorkspaceOp {
    fn from(op: DepsOp) -> Self {
        WorkspaceOp::Deps(op)
    }
}

impl From<MakegenOp> for WorkspaceOp {
    fn from(op: MakegenOp) -> Self {
        WorkspaceOp::Makegen(op)
    }
}

impl From<GistOps> for WorkspaceOp {
    fn from(op: GistOps) -> Self {
        WorkspaceOp::Gist(op)
    }
}

impl From<BootstrapOp> for WorkspaceOp {
    fn from(op: BootstrapOp) -> Self {
        WorkspaceOp::Bootstrap(op)
    }
}

impl From<Buck2Op> for WorkspaceOp {
    fn from(op: Buck2Op) -> Self {
        WorkspaceOp::Buck2(op)
    }
}

impl From<CliToolOp> for WorkspaceOp {
    fn from(op: CliToolOp) -> Self {
        WorkspaceOp::Clippy(op)
    }
}

impl From<LanguageOp> for WorkspaceOp {
    fn from(op: LanguageOp) -> Self {
        WorkspaceOp::Language(op)
    }
}

impl From<PrimitiveOp> for WorkspaceOp {
    fn from(op: PrimitiveOp) -> Self {
        WorkspaceOp::Primitive(op)
    }
}

impl From<TransportOps> for WorkspaceOp {
    fn from(op: TransportOps) -> Self {
        WorkspaceOp::Transport(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_op_from_conversions() {
        // Test that From conversions work
        let _: WorkspaceOp = TransportOps::Execute.into();
        let _: WorkspaceOp = DepsOp::LoadToolRegistry.into();
    }

    #[test]
    fn test_workspace_op_default() {
        let op = WorkspaceOp::default();
        assert!(matches!(op, WorkspaceOp::Transport(TransportOps::Execute)));
    }
}
