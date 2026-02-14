//! Workspace binary invocation mapping.
//!
//! This is the single source of truth for how repo-local binaries are
//! invoked via cargo (package + bin name). Centralizing this mapping
//! prevents drift when binaries move between packages.

use gunbc_ir::CargoInvocation;

/// Repo-local workspace binaries (all live in gunbc-dag).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkspaceBinary {
    Build,
    Bootstrap,
    Ci,
    Codegen,
    CodegenDag,
    DepsConfig,
    Docgen,
    Makegen,
    Pragma,
    Testgen,
}

impl WorkspaceBinary {
    /// Component name used to compose the binary name.
    pub fn component(self) -> &'static str {
        match self {
            WorkspaceBinary::Build => "build",
            WorkspaceBinary::Bootstrap => "bootstrap",
            WorkspaceBinary::Ci => "ci",
            WorkspaceBinary::Codegen => "codegen",
            WorkspaceBinary::CodegenDag => "codegen-dag",
            WorkspaceBinary::DepsConfig => "deps-config",
            WorkspaceBinary::Docgen => "docgen",
            WorkspaceBinary::Makegen => "makegen",
            WorkspaceBinary::Pragma => "pragma",
            WorkspaceBinary::Testgen => "testgen",
        }
    }

    /// Cargo invocation for this workspace binary.
    pub fn invocation(self) -> CargoInvocation {
        CargoInvocation::composed(self.component(), "dag")
    }

    /// Full `cargo run ...` command string.
    pub fn command(self) -> String {
        self.invocation().command()
    }
}
