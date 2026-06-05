//! Transient measurement: does declaration-identity reject cross-call between
//! two structurally-identical distinct named record types?
//! DELETE after measurement — not a permanent test.

use v3_compiler::dag::{Behavior, PortState};
use v3_compiler::diagnostics::Diagnostic;
use v3_compiler::{compile_to_dag, CompileError, Dag};

fn cross_call_rejects(source: &str, file: &str) -> (bool, Vec<String>) {
    let dag = match compile_to_dag(source, file) {
        Ok(dag) => {
            let diags: Vec<String> = dag
                .diagnostics()
                .iter()
                .map(|(_, d)| format!("{d:?}"))
                .collect();
            return (false, diags);
        }
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => {
            return (true, vec![format!("CompileError: {other:?}")]);
        }
    };

    let mut messages: Vec<String> = dag
        .diagnostics()
        .iter()
        .map(|(_, d)| format!("{d:?}"))
        .collect();

    let has_type_mismatch = dag.diagnostics().iter().any(|(_, d)| {
        matches!(
            d,
            Diagnostic::TypeMismatch { .. }
                | Diagnostic::ResolveError { .. }
        )
    });

    let call_unresolved = dag.nodes().iter().any(|node| {
        let Behavior::Transform(t) = node else {
            return false;
        };
        matches!(dag.port(t.output).state(), PortState::Unresolved)
            && dag.diagnostics().get(t.output).is_some()
    });

    (
        has_type_mismatch || call_unresolved,
        messages,
    )
}

#[test]
fn measure_distinct_named_records_reject_cross_call() {
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
    let (rejected, diags) = cross_call_rejects(source, "nominal_distinct_records.v3");
    eprintln!("RECORD_CROSS_CALL rejected={rejected} diags={diags:?}");
    assert!(
        rejected,
        "distinct named records WrapA/WrapB must reject cross-call; got OK. diags={diags:?}"
    );
}

#[test]
fn measure_branded_aliases_reject_cross_call() {
    let source = r#"
type IntentId = String where brand("IntentId")
type IssueId = String where brand("IssueId")

fn expects_intent(x: IntentId) -> String = x

fn call_site(i: IssueId) -> String = expects_intent(i)
"#;
    let (rejected, diags) = cross_call_rejects(source, "nominal_distinct_brands.v3");
    eprintln!("BRAND_ALIAS_CROSS_CALL rejected={rejected} diags={diags:?}");
    assert!(
        rejected,
        "branded aliases IntentId/IssueId must reject cross-call; got OK. diags={diags:?}"
    );
}

#[test]
fn measure_same_named_record_accepts_cross_call_control() {
    let source = r#"
type WrapA {
  value: Int
}

fn expects_a(x: WrapA) -> Int = x.value

fn call_site(a: WrapA) -> Int = expects_a(a)
"#;
    let result = compile_to_dag(source, "nominal_same_record_control.v3");
    eprintln!("SAME_RECORD_CONTROL result={result:?}");
    assert!(
        result.is_ok(),
        "same-type control must compile; got {result:?}"
    );
}
