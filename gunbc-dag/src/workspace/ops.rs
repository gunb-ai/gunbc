//! WorkspaceOp: Unified operation enum for fractal DAG composition.
//!
//! This enum wraps all domain operations, primitives, and transport ops
//! into a single type that can be used throughout the workspace DAG.

use gunbc_exec::{ExecError, Executable, IntoExecResult};
use gunbc_ir::Value;
use std::collections::HashMap;

// Domain ops - local (repo-specific)
use crate::bootstrap::BootstrapOp;
use crate::build::BuildOp;
use crate::ci::CIOp;
use crate::codegen::CodegenOp;
use crate::dag_viz::DagVizGraphOp;
use crate::docgen::DocgenOp;
use crate::makegen::MakegenOp;
use crate::pragma::PragmaOp;
use crate::testgen_dag::TestgenOp;

// Domain ops - external (general tools)
use gunbc_clippy::CliToolOp;
use gunbc_deps::{DepsOp, PlatformEnv};
use gunbc_gist::GistOps;
use gunbc_ir::LanguageOp;
use gunbc_lib_blob::BlobOps;
use gunbc_lib_cloud_ops::CloudEnvStatus;

// Infrastructure ops
use gunbc_lib_transport::cli::execute_cli_tool_op_with_inputs;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{FsEnv, PrimitiveOp};

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
    /// Build workflow operations
    Build(BuildOp),
    /// CI workflow operations
    Ci(CIOp),
    /// Codegen workflow operations
    Codegen(CodegenOp),
    /// Docgen workflow operations
    Docgen(DocgenOp),
    /// Dependency management operations
    Deps(DepsOp),
    /// Dependency platform environment
    DepsEnv(PlatformEnv),
    /// Makefile generation operations
    Makegen(MakegenOp),
    /// Gist operations
    Gist(GistOps),
    /// Bootstrap operations
    Bootstrap(BootstrapOp),
    /// Pragma operations
    Pragma(PragmaOp),
    /// Test generation operations
    Testgen(TestgenOp),
    /// DAG visualization operations
    DagViz(DagVizGraphOp),
    /// Clippy/CLI tool operations
    Clippy(CliToolOp),
    /// Cloud environment status (resource acquisition)
    CloudEnv(CloudEnvStatus),
    /// Filesystem environment (resource acquisition)
    FsEnv(FsEnv),

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
    /// Blob operations (content compare/hash)
    Blob(BlobOps),
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
            WorkspaceOp::Build(op) => op.execute(inputs),
            WorkspaceOp::Ci(op) => op.execute(inputs),
            WorkspaceOp::Codegen(op) => op.execute(inputs),
            WorkspaceOp::Docgen(op) => op.execute(inputs),
            WorkspaceOp::Deps(op) => op.execute(inputs),
            WorkspaceOp::DepsEnv(op) => op.execute(inputs),
            WorkspaceOp::Makegen(op) => op.execute(inputs),
            WorkspaceOp::Gist(op) => op.execute(inputs),
            WorkspaceOp::Bootstrap(op) => op.execute(inputs),
            WorkspaceOp::Pragma(op) => op.execute(inputs),
            WorkspaceOp::Testgen(op) => op.execute(inputs),
            WorkspaceOp::DagViz(op) => op.execute(inputs),
            // CliToolOp execution lives in the transport layer.
            // Transport variant delegates to TransportOps; others to cli execute.
            WorkspaceOp::Clippy(CliToolOp::Transport) => TransportOps::Execute.execute(inputs),
            WorkspaceOp::Clippy(op) => {
                execute_cli_tool_op_with_inputs(op, &inputs).exec_context("CliToolOp error")
            }
            WorkspaceOp::CloudEnv(op) => op.execute(inputs),
            WorkspaceOp::FsEnv(op) => op.execute(inputs),
            // Language ops
            WorkspaceOp::Language(_op) => {
                // LanguageOp nodes are mostly config nodes - return empty for now
                // In the future, this could dispatch to language-specific execution
                Ok(HashMap::new())
            }
            // Infrastructure ops
            WorkspaceOp::Primitive(op) => op.execute(inputs),
            WorkspaceOp::Blob(op) => op.execute(inputs),
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

impl From<BuildOp> for WorkspaceOp {
    fn from(op: BuildOp) -> Self {
        WorkspaceOp::Build(op)
    }
}

impl From<CodegenOp> for WorkspaceOp {
    fn from(op: CodegenOp) -> Self {
        WorkspaceOp::Codegen(op)
    }
}

impl From<DocgenOp> for WorkspaceOp {
    fn from(op: DocgenOp) -> Self {
        WorkspaceOp::Docgen(op)
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

impl From<CliToolOp> for WorkspaceOp {
    fn from(op: CliToolOp) -> Self {
        WorkspaceOp::Clippy(op)
    }
}

impl From<PragmaOp> for WorkspaceOp {
    fn from(op: PragmaOp) -> Self {
        WorkspaceOp::Pragma(op)
    }
}

impl From<TestgenOp> for WorkspaceOp {
    fn from(op: TestgenOp) -> Self {
        WorkspaceOp::Testgen(op)
    }
}

impl From<DagVizGraphOp> for WorkspaceOp {
    fn from(op: DagVizGraphOp) -> Self {
        WorkspaceOp::DagViz(op)
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

impl From<BlobOps> for WorkspaceOp {
    fn from(op: BlobOps) -> Self {
        WorkspaceOp::Blob(op)
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
