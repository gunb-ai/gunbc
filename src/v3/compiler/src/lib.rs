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
pub mod lens_unused_parameters;
pub mod operators;
pub mod serialize;
pub mod types;

mod bootstrap;
mod infer;
mod lower;
mod parse;
mod tokenize;

pub use dag::Dag;
pub use diagnostics::{Diagnostic, SourceSpan};
pub use emit_rust::EmitError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageSnapshotKind {
    Text,
    Dag,
}

#[derive(Debug, Clone)]
pub struct StageSnapshot {
    pub stage: &'static str,
    pub kind: StageSnapshotKind,
    pub bytes: Vec<u8>,
    pub dag: Option<Dag>,
}

#[derive(Debug)]
pub enum StageSnapshotError {
    Compile(CompileError),
    Emit(emit_rust::EmitError),
}

#[derive(Debug)]
pub struct FixedPointMismatch {
    pub stage: String,
    pub detail: String,
}

/// Test-only hook: tokenize a source string. Used by the
/// `real_stdlib_parse_smoke` integration test to verify the parser
/// accepts production `dsl/std/*.dag` files before bootstrap migration.
#[doc(hidden)]
pub fn tokenize_for_test(source: &str, file: &str) -> Result<Vec<tokenize::Token>, Diagnostic> {
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

pub fn default_fixed_point_source() -> &'static str {
    "let x: Int = 1 + 2\nlet y: Int = x + 3\n"
}

pub fn compile_stage_snapshots(
    source: &str,
    file: &str,
) -> Result<Vec<StageSnapshot>, StageSnapshotError> {
    let tokens = tokenize::tokenize(source, file)
        .map_err(CompileError::Tokenize)
        .map_err(StageSnapshotError::Compile)?;
    let surface = parse::parse(&tokens, file)
        .map_err(CompileError::Parse)
        .map_err(StageSnapshotError::Compile)?;
    let parse_bytes = format!("{surface:#?}").into_bytes();

    let mut lower_dag = lower::lower(&surface);
    let lower_snapshot = lower_dag.clone();
    let lower_bytes = serialize::serialize_dag(&lower_snapshot);

    infer::infer(&mut lower_dag);
    if !lower_dag.diagnostics().is_empty() {
        return Err(StageSnapshotError::Compile(CompileError::Semantic(
            lower_dag.clone(),
        )));
    }

    let infer_snapshot = lower_dag.clone();
    let infer_bytes = serialize::serialize_dag(&infer_snapshot);
    let emitted = emit_rust::emit_rust(&lower_dag).map_err(StageSnapshotError::Emit)?;

    Ok(vec![
        StageSnapshot {
            stage: "parse",
            kind: StageSnapshotKind::Text,
            bytes: parse_bytes,
            dag: None,
        },
        StageSnapshot {
            stage: "lower",
            kind: StageSnapshotKind::Dag,
            bytes: lower_bytes,
            dag: Some(lower_snapshot),
        },
        StageSnapshot {
            stage: "infer",
            kind: StageSnapshotKind::Dag,
            bytes: infer_bytes.clone(),
            dag: Some(infer_snapshot.clone()),
        },
        StageSnapshot {
            // Ownership Phase 1 lands in a parallel workstream. Until the
            // stage materializes its own Dag facts, the fixed-point harness
            // snapshots the post-infer Dag at the declared boundary.
            stage: "compute_ownership",
            kind: StageSnapshotKind::Dag,
            bytes: infer_bytes,
            dag: Some(infer_snapshot),
        },
        StageSnapshot {
            stage: "emit",
            kind: StageSnapshotKind::Text,
            bytes: emitted.into_bytes(),
            dag: None,
        },
    ])
}

pub fn compare_stage_snapshots(
    lhs: &[StageSnapshot],
    rhs: &[StageSnapshot],
) -> Result<(), FixedPointMismatch> {
    if lhs.len() != rhs.len() {
        return Err(FixedPointMismatch {
            stage: "pipeline".to_string(),
            detail: format!(
                "stage count mismatch: pass1 has {}, pass2 has {}",
                lhs.len(),
                rhs.len()
            ),
        });
    }

    for (left, right) in lhs.iter().zip(rhs.iter()) {
        if left.stage != right.stage {
            return Err(FixedPointMismatch {
                stage: "pipeline".to_string(),
                detail: format!(
                    "stage order mismatch: pass1 has `{}`, pass2 has `{}`",
                    left.stage, right.stage
                ),
            });
        }
        if left.bytes == right.bytes {
            continue;
        }

        let detail = match (&left.dag, &right.dag) {
            (Some(lhs_dag), Some(rhs_dag)) => serialize::first_difference(lhs_dag, rhs_dag)
                .map(|diff| diff.detail)
                .unwrap_or_else(|| first_differing_line(&left.bytes, &right.bytes)),
            _ => first_differing_line(&left.bytes, &right.bytes),
        };
        return Err(FixedPointMismatch {
            stage: left.stage.to_string(),
            detail,
        });
    }

    Ok(())
}

fn first_differing_line(lhs: &[u8], rhs: &[u8]) -> String {
    let lhs = String::from_utf8_lossy(lhs);
    let rhs = String::from_utf8_lossy(rhs);
    for (idx, (left, right)) in lhs.lines().zip(rhs.lines()).enumerate() {
        if left != right {
            return format!(
                "first differing line {}: pass1=`{}`, pass2=`{}`",
                idx + 1,
                left,
                right
            );
        }
    }
    format!(
        "snapshot byte-length mismatch: pass1={} bytes, pass2={} bytes",
        lhs.len(),
        rhs.len()
    )
}
