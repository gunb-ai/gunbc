//! Universal capability types for compilation and codegen (WF14/WF15).
//!
//! These capabilities are shared across all tool workflows via the global
//! ledger. The planner resolves freshness from content-addressed keys,
//! bypassing `cargo run` on warm state.

use serde::{Deserialize, Serialize};

// ============================================================================
// Compilation Capability (WF14)
// ============================================================================

/// Two-phase compilation model.
///
/// Bootstrap-safe binaries (codegen, ci) compile without generated sources.
/// Tool binaries depend on codegen outputs. The planner manages both phases
/// as keyed units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompilationPhase {
    /// Bootstrap phase: compiles without codegen dependency.
    /// Binaries: gunbc-codegen, gunbc-ci.
    Bootstrap,
    /// Tool phase: depends on codegen outputs.
    /// Binaries: all tool binaries (gist, bootstrap, makegen, etc.)
    Tool,
}

impl CompilationPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompilationPhase::Bootstrap => "bootstrap",
            CompilationPhase::Tool => "tool",
        }
    }
}

impl std::fmt::Display for CompilationPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Typed cache miss reasons for compilation capability.
///
/// Each variant maps to a specific invalidation signal from the key contract:
/// - `source_hashes`: content hashes of all `*.rs` + `Cargo.toml` in dep tree
/// - `cargo_metadata_hash`: hash of cargo metadata dependency graph
/// - `compiler_version`: `rustc --version` output hash
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompilationMissReason {
    /// Source file(s) changed since last build.
    SourceChanged { changed_crate: String },
    /// Cargo dependency graph changed (version bump, new dep, etc.).
    DependencyChanged { changed_dep: String },
    /// Rust compiler version changed (toolchain update).
    CompilerChanged { old: String, new: String },
    /// Binary has never been built (first run).
    NeverBuilt,
}

impl std::fmt::Display for CompilationMissReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationMissReason::SourceChanged { changed_crate } => {
                write!(f, "source-changed:{changed_crate}")
            }
            CompilationMissReason::DependencyChanged { changed_dep } => {
                write!(f, "dependency-changed:{changed_dep}")
            }
            CompilationMissReason::CompilerChanged { old, new } => {
                write!(f, "compiler-changed:{old}->{new}")
            }
            CompilationMissReason::NeverBuilt => write!(f, "never-built"),
        }
    }
}

/// Key fields for compilation materialization key.
pub mod compilation_key {
    /// Port name for source hashes input.
    pub const SOURCE_HASHES: &str = "source_hashes";
    /// Port name for cargo metadata hash input.
    pub const CARGO_METADATA_HASH: &str = "cargo_metadata_hash";
    /// Port name for compiler version hash input.
    pub const COMPILER_VERSION: &str = "compiler_version";
    /// Port name for compilation phase input.
    pub const COMPILATION_PHASE: &str = "compilation_phase";
    /// Port name for compilation profile input.
    pub const PROFILE: &str = "profile";
    /// Port name for target triple input.
    pub const TARGET: &str = "target";
    /// Port name for binary paths output.
    pub const BINARY_PATHS: &str = "binary_paths";
}

// ============================================================================
// Codegen Capability (WF15)
// ============================================================================

/// Typed cache miss reasons for codegen capability.
///
/// Key contract:
/// - `dsl_source_hashes`: content hashes of `dsl/**/*.dag` files
/// - `codegen_binary_version`: semantic version from codegen binary manifest
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodegenMissReason {
    /// DSL source file(s) changed since last codegen run.
    DslSourceChanged { changed_file: String },
    /// Codegen binary itself changed (logic update).
    CodegenBinaryChanged {
        old_version: String,
        new_version: String,
    },
    /// Codegen has never run (first invocation).
    NeverRun,
}

impl std::fmt::Display for CodegenMissReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenMissReason::DslSourceChanged { changed_file } => {
                write!(f, "dsl-source-changed:{changed_file}")
            }
            CodegenMissReason::CodegenBinaryChanged {
                old_version,
                new_version,
            } => write!(f, "codegen-binary-changed:{old_version}->{new_version}"),
            CodegenMissReason::NeverRun => write!(f, "never-run"),
        }
    }
}

/// Key fields for codegen materialization key.
pub mod codegen_key {
    /// Port name for DSL source hashes input.
    pub const DSL_SOURCE_HASHES: &str = "dsl_source_hashes";
    /// Port name for codegen binary version input.
    pub const CODEGEN_BINARY_VERSION: &str = "codegen_binary_version";
    /// Port name for codegen freshness output.
    pub const CODEGEN_FRESH: &str = "codegen_fresh";
}

// ============================================================================
// Canonical Process IDs
// ============================================================================

/// Canonical process ID for the compilation capability.
///
/// Both `ci.compilation.ensure` and `gist.compilation.ensure` resolve to
/// the same `WorkIdentity` via this process ID.
pub const COMPILATION_PROCESS_ID: &str = "compilation";

/// Canonical process ID for the codegen capability.
///
/// Both `ci.codegen.ensure` and `gist.codegen.ensure` resolve to
/// the same `WorkIdentity` via this process ID.
pub const CODEGEN_PROCESS_ID: &str = "codegen";

/// Canonical unit ID for compilation ensure operations.
///
/// Uses underscore (not dot) so `canonicalize_unit_id` preserves the full
/// name. Dot-separated IDs get their first segment stripped (designed for
/// `ci.codegen` → `codegen`), which would cause `compilation.ensure` and
/// `codegen.ensure` to both canonicalize to `ensure` (collision).
pub const COMPILATION_ENSURE_UNIT: &str = "compilation_ensure";

/// Canonical unit ID for codegen ensure operations.
///
/// See [`COMPILATION_ENSURE_UNIT`] for naming rationale.
pub const CODEGEN_ENSURE_UNIT: &str = "codegen_ensure";

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compilation_phase_display() {
        assert_eq!(CompilationPhase::Bootstrap.to_string(), "bootstrap");
        assert_eq!(CompilationPhase::Tool.to_string(), "tool");
    }

    #[test]
    fn compilation_miss_reason_display() {
        assert_eq!(
            CompilationMissReason::SourceChanged {
                changed_crate: "gunbc-ir".to_string()
            }
            .to_string(),
            "source-changed:gunbc-ir"
        );
        assert_eq!(CompilationMissReason::NeverBuilt.to_string(), "never-built");
    }

    #[test]
    fn codegen_miss_reason_display() {
        assert_eq!(
            CodegenMissReason::DslSourceChanged {
                changed_file: "dsl/tools/gist.dag".to_string()
            }
            .to_string(),
            "dsl-source-changed:dsl/tools/gist.dag"
        );
        assert_eq!(CodegenMissReason::NeverRun.to_string(), "never-run");
    }

    #[test]
    fn compilation_miss_reason_serializes() {
        let reason = CompilationMissReason::CompilerChanged {
            old: "1.80.0".to_string(),
            new: "1.81.0".to_string(),
        };
        let json = serde_json::to_string(&reason).expect("serialize");
        let deserialized: CompilationMissReason = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(reason, deserialized);
    }

    #[test]
    fn codegen_miss_reason_serializes() {
        let reason = CodegenMissReason::CodegenBinaryChanged {
            old_version: "0.1.0".to_string(),
            new_version: "0.2.0".to_string(),
        };
        let json = serde_json::to_string(&reason).expect("serialize");
        let deserialized: CodegenMissReason = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(reason, deserialized);
    }
}
