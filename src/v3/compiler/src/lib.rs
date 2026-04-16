// v3 compiler — M0 substrate skeleton.
//
// Pipeline (target end state for M0):
//   source text -> tokenize -> parse -> lower to L1 behaviors -> infer -> Dag
//
// Fail-closed compile boundary (invariant C-8):
//   compile_to_dag returns Ok(Dag) ONLY when the diagnostic table
//   is empty. Any semantic errors (type mismatches, unresolved
//   names, arity errors, etc.) surface as Err(CompileError::Semantic(dag))
//   — the dag is still handed back so the caller can inspect the
//   diagnostics, but the Result variant is Err.
//
//   Structural errors (tokenize/parse) surface as their own variants
//   because they occur before a Dag exists. G5: no TypeError variant
//   on CompileError — type errors live on the Dag, not in the Err
//   payload.

pub mod dag;
pub mod diagnostics;
pub mod emit_rust;
pub mod lens_cost;
pub mod lens_depth;
pub mod lens_provenance;
pub mod lens_testgen;
pub mod lens_unused_parameters;
pub mod operators;
pub mod types;

mod bootstrap;
mod infer;
mod lower;
mod parse;
mod tokenize;

pub use dag::Dag;
pub use diagnostics::{Diagnostic, SourceSpan};

/// Test-only hook: tokenize a source string. Used by the
/// `real_stdlib_parse_smoke` integration test to verify the parser
/// accepts production `dsl/std/*.dag` files before bootstrap migration.
#[doc(hidden)]
pub fn tokenize_for_test(
    source: &str,
    file: &str,
) -> Result<Vec<tokenize::Token>, Diagnostic> {
    tokenize::tokenize(source, file)
}

/// Test-only hook: parse a token stream into a surface module.
#[doc(hidden)]
pub fn parse_for_test(
    tokens: &[tokenize::Token],
    file: &str,
) -> Result<parse::SurfaceModule, Diagnostic> {
    parse::parse(tokens, file)
}

/// Top-level compile failure. Distinguishes three structural
/// categories of failure by phase of the pipeline where they occurred.
///
/// **Dissolution receipt: TERMINAL.** Three variants, each with a
/// structurally distinct payload:
/// - `Tokenize(Diagnostic)`: tokenization produced a single diagnostic;
///   no Dag exists yet, so no Dag payload.
/// - `Parse(Diagnostic)`: parsing produced a single diagnostic; no Dag
///   exists yet.
/// - `Semantic(Dag)`: lowering/inference produced one or more
///   diagnostics; the Dag exists and carries them in its diagnostic
///   table, so it's handed back as the payload for caller inspection.
///
/// The three variants correspond to three structurally different
/// failure states (no-Dag-yet with a diagnostic vs Dag-with-
/// diagnostic-table). Pattern 2 (variant-is-data) fails because the
/// payloads are different types. Pattern 3 (algebraic-form) doesn't
/// apply — these are failure phases, not algebraic operations.
///
/// Guardrail G5: there is no `TypeError` variant. Type errors are
/// data on the Dag via the diagnostic table, not fields on the
/// error type. `Semantic(Dag)` is a handoff, not a classification of
/// what went wrong — the caller reads `dag.diagnostics()` for
/// specifics. This is what "fail-closed at the boundary" means in
/// practice: a successful compile returns `Ok(Dag)` with an empty
/// diagnostic table; a failed compile returns `Err(Semantic(Dag))`
/// with a non-empty one. There is no third outcome.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum CompileError {
    Tokenize(diagnostics::Diagnostic),
    Parse(diagnostics::Diagnostic),
    /// Semantic errors. The Dag is included so callers can inspect
    /// `dag.diagnostics()` to see what went wrong. `Err(Semantic(_))`
    /// means: the compile reached infer, some (>=1) diagnostics were
    /// produced, and the result is not usable.
    Semantic(Dag),
}

// `result_large_err`: clippy flags `Result<Dag, CompileError>`
// because `CompileError::Semantic(Dag)` carries a `Dag` payload
// (~264 bytes after the M1(3) PR-B-unwind R1 added the realization
// meta cache). Boxing the Dag would touch every pattern-match
// against `CompileError::Semantic` in the test suite, and the
// payload is on the cold failure path where the indirection would
// matter less than the API churn. Targeted `allow` on the function
// signature only — the rest of the crate keeps the lint enforced.
#[allow(clippy::result_large_err)]
pub fn compile_to_dag(source: &str, file: &str) -> Result<Dag, CompileError> {
    let tokens = tokenize::tokenize(source, file).map_err(CompileError::Tokenize)?;
    let surface = parse::parse(&tokens, file).map_err(CompileError::Parse)?;
    let mut dag = lower::lower(&surface);
    infer::infer(&mut dag);
    if dag.diagnostics().is_empty() {
        Ok(dag)
    } else {
        Err(CompileError::Semantic(dag))
    }
}
