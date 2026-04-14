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
pub mod lens_depth;
pub mod lens_provenance;
pub mod types;

mod infer;
mod lower;
mod parse;
mod tokenize;

pub use dag::Dag;
pub use diagnostics::{Diagnostic, SourceSpan};

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
