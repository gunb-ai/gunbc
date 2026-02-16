//! daglang-emit: CodegenBackend trait and Rust backend.
//!
//! The final compiler phase: emit runnable code from GraphIR + derived
//! metadata. Each backend implements `CodegenBackend` to produce
//! target-language code.
//!
//! # Pipeline position
//!
//! ```text
//! GraphIR + ProgressManifest + TestObligations
//!   → [daglang-emit] → Rust source files (Phase 1)
//!                     → Go source files (Phase 4)
//! ```
//!
//! # What gets emitted per module
//!
//! ```text
//! tools/makegen.dag
//!   ├── types/      Type definitions (records, enums)
//!   ├── fn/         Pure functors → target language functions
//!   ├── transport/  Transport wiring (HTTP, shell, file)
//!   ├── func/       DAG orchestrator (topo-scheduled execution)
//!   ├── cli/        CLI entrypoint (arg parsing from func inputs)
//!   ├── test/       Test harness (4-bucket obligations)
//!   ├── mock/       MockSpec (from service declarations)
//!   ├── manifest/   ProgressManifest (static, from topology)
//!   └── makefile/   Makefile target (from module metadata)
//! ```

/// The codegen backend trait. Each target language implements this.
pub trait CodegenBackend {
    /// Emit a type definition (record, enum, alias).
    fn emit_type(&self, ty: &str) -> String;

    /// Emit a pure functor as a target-language function.
    fn emit_fn(&self, name: &str) -> String;

    /// Emit a DAG orchestrator for an effectful function.
    fn emit_func(&self, name: &str) -> String;

    /// Emit transport wiring (HTTP client, shell exec, file I/O).
    fn emit_transport(&self, spec: &str) -> String;

    /// Emit a test harness from test obligations.
    fn emit_test(&self, obligation: &str) -> String;

    /// Emit CLI entrypoint from DAG entry ports.
    fn emit_cli(&self, entrypoints: &[String]) -> String;

    /// Emit a progress manifest (static topology for renderers).
    fn emit_progress_manifest(&self, manifest: &str) -> String;
}

/// Errors during emission.
#[derive(Debug)]
pub enum EmitError {
    /// A construct couldn't be emitted for the target backend.
    UnsupportedConstruct { backend: String, construct: String },
}
