// v3 compiler — M0 substrate skeleton.
//
// Pipeline (target end state for M0 Test 1):
//   source text -> tokenize -> parse -> lower to L1 behaviors -> infer -> Dag
//
// Type errors do NOT flow through CompileError — they go to the
// DiagnosticTable via mark_unresolved. CompileError covers structural
// tokenize / parse failures only. Guardrail G5.

pub mod dag;
pub mod diagnostics;
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
}

pub fn compile_to_dag(source: &str, file: &str) -> Result<Dag, CompileError> {
    let tokens = tokenize::tokenize(source, file).map_err(CompileError::Tokenize)?;
    let surface = parse::parse(&tokens, file).map_err(CompileError::Parse)?;
    let mut dag = lower::lower(&surface);
    infer::infer(&mut dag);
    Ok(dag)
}
