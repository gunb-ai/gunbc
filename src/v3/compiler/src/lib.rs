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
pub mod emit_go;
pub mod emit_python;
pub mod emit_rust;
pub mod lens_depth;
pub mod lens_testgen;
pub mod lens_unused_parameters;
pub mod operators;
pub mod post_emit_verifier;
pub mod serialize;
pub mod types;

/// Cost lens. The authority lives in `src/v3/lenses/complexity.dag`;
/// the Rust projection is auto-emitted into
/// `src/v3/compiler/src/lens_cost_generated.rs` and re-exported here
/// so callers use `v3_compiler::lens_cost::{cost_of, CostLookup}`.
/// Editing the lens means editing the `.dag` — there is no
/// hand-written implementation on this crate side.
///
/// L-8 compliance: `cost_of` returns the typed `CostLookup` carrier
/// (`MissingCost | FoundCost(Int)`). Callers pattern-match on the
/// variant rather than receiving a panicked-collapsed `usize`.
pub mod lens_cost {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_cost_generated.rs");
    }

    pub use generated::{cost_of, CostLookup};
}

/// Provenance lens. The authority lives in
/// `src/v3/lenses/provenance.dag`; the Rust projection is auto-emitted
/// into `src/v3/compiler/src/lens_provenance_generated.rs` and wrapped
/// here as a module so callers use `v3_compiler::lens_provenance`.
/// Editing the lens means editing the `.dag` — there is no hand-written
/// implementation on this crate side.
///
/// Only `Origin` and `origin_of` are re-exported. The generated module
/// also declares internal helper carriers (`PortLookup`,
/// `BehaviorLookup`) and their `find_*` / `behavior_id` walkers, which
/// exist solely because the substrate still exposes `Dag.ports` /
/// `Dag.nodes` as linear lists. Those helpers are bounded scaffolding
/// that dissolves when the substrate grows total keyed `port(id)` /
/// `node(id)` accessors — keeping them crate-private now prevents the
/// tracked-scaffold from leaking into `v3_compiler::lens_provenance`'s
/// public surface and attracting downstream consumers.
pub mod lens_provenance {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_provenance_generated.rs");
    }

    pub use generated::{origin_of, Origin};
}

/// Structural-resolution lens. The authority lives in
/// `src/v3/lenses/structural_resolution.dag`; the Rust projection is
/// auto-emitted into `src/v3/compiler/src/lens_structural_resolution_generated.rs`
/// and wrapped here as a module so callers use
/// `v3_compiler::lens_structural_resolution`. Editing the lens means
/// editing the `.dag` — there is no hand-written implementation on
/// this crate side.
///
/// Detects leaked `ArrowBody::Pending` on named user Declarations.
/// Defense-in-depth regression pin for the R13 fix (see the `.dag`
/// source for the full detection rule and disposal trigger).
pub mod lens_structural_resolution {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_structural_resolution_generated.rs");
    }

    pub use generated::{check, UnresolvedArrowBody};
}

mod bootstrap;
mod infer;
mod lower;
mod parse;
mod pipeline_authority;
mod tokenize;
mod workflow_idempotency;

pub use dag::Dag;
pub use diagnostics::{Diagnostic, SourceSpan};
pub use emit_rust::EmitError;
pub use workflow_idempotency::{analyze_workflow, compose_operation_effects, operation_to_breaker};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageSnapshotKind {
    Surface,
    Text,
    Dag,
}

#[derive(Debug, Clone)]
pub struct StageSnapshot {
    pub stage: String,
    pub kind: StageSnapshotKind,
    pub bytes: Vec<u8>,
    pub dag: Option<Dag>,
}

#[derive(Debug)]
pub enum StageSnapshotError {
    Compile(Box<CompileError>),
    Emit(Box<emit_rust::EmitError>),
    Pipeline(String),
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

/// Test-only hook: inject a named Arrow declaration with
/// `ArrowBody::Pending` directly into `dag`. Synthesizes the exact
/// shape that `lens_structural_resolution` is designed to flag —
/// the "named user fn seeded with Pending and never patched" shape
/// that `lower_fn_item` forbids but that a future regression in the
/// body-patching path could re-introduce (see the R13 fix in
/// `lower.rs:2293` for the historical precedent). Lives here and
/// calls the `pub(crate)` `alloc_declaration_id` / `push_declaration`
/// primitives directly so that this narrow "inject one named
/// Arrow(Pending)" form is the only public construction path —
/// exposing the raw primitives would widen the mutation surface
/// beyond what the lens's synthetic-Dag test needs.
#[doc(hidden)]
pub fn inject_named_pending_arrow_for_test(
    dag: &mut Dag,
    name: &str,
    output_type: dag::DeclarationId,
) -> dag::DeclarationId {
    let id = dag.alloc_declaration_id();
    dag.push_declaration(dag::Declaration {
        id,
        name: Some(name.to_string()),
        connective: dag::TypeConnective::Arrow {
            inputs: Vec::new(),
            output: output_type,
            body: dag::ArrowBody::Pending,
        },
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        span: diagnostics::SourceSpan::new("test", 0, 0),
    });
    id
}

/// Test-only hook: parse a token stream into a surface module.
#[doc(hidden)]
pub fn parse_for_test(
    tokens: &[tokenize::Token],
    file: &str,
) -> Result<parse::SurfaceModule, Diagnostic> {
    parse::parse(tokens, file)
}

/// Test hook: pipeline stage identifiers in `compile { ... }` order in
/// `pipeline.dag` — the same ordering as `materialize_pipeline_realizations`.
#[doc(hidden)]
pub fn pipeline_compile_order_stage_names() -> Result<Vec<String>, String> {
    pipeline_authority::pipeline_compile_order_names()
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
    let pipeline_dag = Dag::new();
    if !pipeline_dag.diagnostics().is_empty() {
        return Err(StageSnapshotError::Compile(Box::new(
            CompileError::Semantic(pipeline_dag),
        )));
    }
    let pipeline = pipeline_authority::ordered_pipeline_stages(&pipeline_dag)
        .map_err(StageSnapshotError::Pipeline)?;

    let tokens = tokenize::tokenize(source, file)
        .map_err(CompileError::Tokenize)
        .map_err(|error| StageSnapshotError::Compile(Box::new(error)))?;
    let surface = parse::parse(&tokens, file)
        .map_err(CompileError::Parse)
        .map_err(|error| StageSnapshotError::Compile(Box::new(error)))?;
    let parse_bytes = format!("{surface:#?}").into_bytes();

    let mut lower_dag = lower::lower(&surface);
    let lower_snapshot = lower_dag.clone();
    let lower_bytes = serialize::serialize_dag(&lower_snapshot);

    infer::infer(&mut lower_dag);
    if !lower_dag.diagnostics().is_empty() {
        return Err(StageSnapshotError::Compile(Box::new(
            CompileError::Semantic(lower_dag.clone()),
        )));
    }

    let infer_snapshot = lower_dag.clone();
    let infer_bytes = serialize::serialize_dag(&infer_snapshot);
    let emitted = emit_rust::emit_rust(&lower_dag)
        .map_err(Box::new)
        .map_err(StageSnapshotError::Emit)?;

    let mut snapshots = Vec::with_capacity(pipeline.len());
    for stage in pipeline {
        let (kind, bytes, dag) = match stage.stage_name.as_str() {
            "parse" => (StageSnapshotKind::Surface, parse_bytes.clone(), None),
            "lower" => (
                StageSnapshotKind::Dag,
                lower_bytes.clone(),
                Some(lower_snapshot.clone()),
            ),
            "infer" => (
                StageSnapshotKind::Dag,
                infer_bytes.clone(),
                Some(infer_snapshot.clone()),
            ),
            "compute_ownership" => (
                StageSnapshotKind::Dag,
                infer_bytes.clone(),
                Some(infer_snapshot.clone()),
            ),
            "lens_complexity" => (
                StageSnapshotKind::Dag,
                infer_bytes.clone(),
                Some(infer_snapshot.clone()),
            ),
            "emit" => (StageSnapshotKind::Text, emitted.clone().into_bytes(), None),
            other => {
                return Err(StageSnapshotError::Pipeline(format!(
                    "pipeline stage `{other}` has no Rust snapshot implementation"
                )));
            }
        };

        if !snapshot_kind_matches(stage.snapshot_kind, kind) {
            return Err(StageSnapshotError::Pipeline(format!(
                "pipeline stage `{}` declares snapshot kind {:?} but Rust produced {:?}",
                stage.stage_name, stage.snapshot_kind, kind
            )));
        }

        snapshots.push(StageSnapshot {
            stage: stage.stage_name,
            kind,
            bytes,
            dag,
        });
    }

    Ok(snapshots)
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
        if left.kind != right.kind {
            return Err(FixedPointMismatch {
                stage: left.stage.clone(),
                detail: format!(
                    "snapshot kind mismatch at stage `{}`: pass1={:?}, pass2={:?}",
                    left.stage, left.kind, right.kind
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
            stage: left.stage.clone(),
            detail,
        });
    }

    Ok(())
}

fn snapshot_kind_matches(
    declared: pipeline_authority::PipelineSnapshotKind,
    actual: StageSnapshotKind,
) -> bool {
    matches!(
        (declared, actual),
        (
            pipeline_authority::PipelineSnapshotKind::Surface,
            StageSnapshotKind::Surface
        ) | (
            pipeline_authority::PipelineSnapshotKind::Dag,
            StageSnapshotKind::Dag
        ) | (
            pipeline_authority::PipelineSnapshotKind::Text,
            StageSnapshotKind::Text
        )
    )
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
