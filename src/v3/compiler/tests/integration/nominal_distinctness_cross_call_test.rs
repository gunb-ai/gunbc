//! Execution witnesses: distinct named types reject cross-call even when
//! structurally identical (Stage 1 scoping — fresh-nominal-type desugar).
//!
//! Confirms declaration-identity / template-binding machinery rejects
//! `WrapA` at a `WrapB` call site (records) and branded aliases reject via
//! refinement discharge (IntentId/IssueId). Control: same-type call compiles.
//!
//! **P5 receipt (INVARIANTS.md §P5 Mechanism (b) — SG-0 `EXPECTED_HAND_AUTHORED_TEST`):**
//! explicit deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero` (ROADMAP.md:43); Stage-1 fresh-nominal-type
//! disjointness witnesses until `.dag` `TestClaim` coverage executes the same
//! cross-call rejection facts directly. Dissolves when refinement-desugar Stage-2+
//! claim runners replace this hand-Rust harness.

use v3_compiler::dag::{Behavior, PortState};
use v3_compiler::diagnostics::Diagnostic;
use v3_compiler::{compile_to_dag, CompileError, Dag};

fn semantic_dag(source: &str, file: &str) -> Dag {
    match compile_to_dag(source, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("expected parse+lower to reach infer (Semantic), got {other:?}"),
    }
}

fn cross_call_rejects_semantically(source: &str, file: &str) -> Vec<String> {
    let dag = semantic_dag(source, file);
    let messages: Vec<String> = dag
        .diagnostics()
        .iter()
        .map(|(_, d)| format!("{d:?}"))
        .collect();

    let has_resolve_error = dag
        .diagnostics()
        .iter()
        .any(|(_, d)| matches!(d, Diagnostic::ResolveError { .. }));

    let call_unresolved = dag.nodes().iter().any(|node| {
        let Behavior::Transform(t) = node else {
            return false;
        };
        matches!(dag.port(t.output).state(), PortState::Unresolved)
            && dag.diagnostics().get(t.output).is_some()
    });

    assert!(
        has_resolve_error || call_unresolved,
        "cross-call must fail closed with ResolveError or unresolved call port; diags={messages:?}"
    );
    messages
}

#[test]
fn distinct_named_records_reject_cross_call() {
    let source = r#"
type WrapA {
  value: Int
}

type WrapB {
  value: Int
}

fn expects_a(x: WrapA) -> Int = x.value

fn call_site(b: WrapB) -> Int = expects_a(b)
"#;
    let diags = cross_call_rejects_semantically(source, "nominal_distinct_records.v3");
    assert!(
        diags
            .iter()
            .any(|d| d.contains("WrapA") && d.contains("WrapB")),
        "rejection must name both carriers; diags={diags:?}"
    );
    assert!(
        diags.iter().any(|d| d.contains("ResolveError")),
        "records cross-call must surface ResolveError; diags={diags:?}"
    );
}

#[test]
fn branded_aliases_reject_cross_call() {
    let source = r#"
type IntentId = String where brand("IntentId")
type IssueId = String where brand("IssueId")

fn expects_intent(x: IntentId) -> String = x

fn call_site(i: IssueId) -> String = expects_intent(i)
"#;
    let diags = cross_call_rejects_semantically(source, "nominal_distinct_brands.v3");
    assert!(
        diags.iter().any(|d| d.contains("ResolveError")),
        "brand-alias cross-call must surface ResolveError (not parser failure); diags={diags:?}"
    );
    assert!(
        diags.iter().any(|d| d.contains("expects_intent")),
        "rejection must name the callee; diags={diags:?}"
    );
    assert!(
        diags
            .iter()
            .any(|d| d.contains("where") && d.contains("refinement")),
        "brand-alias rejection must be refinement discharge, not unrelated failure; diags={diags:?}"
    );
}

#[test]
fn same_named_record_accepts_cross_call_control() {
    let source = r#"
type WrapA {
  value: Int
}

fn expects_a(x: WrapA) -> Int = x.value

fn call_site(a: WrapA) -> Int = expects_a(a)
"#;
    let result = compile_to_dag(source, "nominal_same_record_control.v3");
    assert!(
        result.is_ok(),
        "same-type control must compile; got {result:?}"
    );
}
