//! **Layer:** integration

use std::path::PathBuf;

use v3_compiler::dag::{FieldValue, LiteralBits};
use v3_compiler::test_runner::TestClaimValue;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::{compile_to_dag, CompileError};

const MOCK_BACKED_INVARIANT_FIXTURE: &str =
    include_str!("../fixtures/r1_mock_backed_invariant_gate.dag");

fn compile_clean(source: &str, file: &str) -> v3_compiler::dag::Dag {
    match compile_to_dag(source, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "{file} should compile cleanly, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(err) => panic!("{file} should compile cleanly, got {err:?}"),
    }
}

fn assert_all_pass(results: &[v3_compiler::test_runner::ClaimEvaluation]) {
    assert!(
        results
            .iter()
            .all(|result| result.result == ClaimResult::Pass),
        "expected every claim to pass, got {results:?}"
    );
}

fn claim_value(dag: &v3_compiler::dag::Dag, name: &str) -> TestClaimValue {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("claim `{name}` not found"));
    TestClaimValue::from_declaration(decl)
        .unwrap_or_else(|reason| panic!("claim `{name}` should lower structurally: {reason}"))
}

#[test]
fn test_runner_runs_compiles_and_fails_with_diagnostic_claims() {
    let source = r#"
data claim_compiles: TestClaim = {
  name: "compiles",
  source: "let x: Int = 1",
  file_name: "runner_compiles.v3",
  predicate: Compiles,
  requires: []
}

data claim_fails: TestClaim = {
  name: "fails with type mismatch",
  source: "let x: Bool = 1",
  file_name: "runner_fails.v3",
  predicate: FailsWithDiagnostic({ kind: TypeMismatch, detail_contains: AnyDetail }),
  requires: []
}

data suite: TestSuite = {
  name: "runner_smoke",
  claims: [claim_compiles, claim_fails]
}
"#;
    let dag = compile_clean(source, "test_runner_smoke.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 2);
    assert_all_pass(&results);
}

#[test]
fn test_runner_reports_claim_failures() {
    let source = r#"
data claim_should_compile: TestClaim = {
  name: "bad compile expectation",
  source: "let x: Bool = 1",
  file_name: "runner_bad_compile.v3",
  predicate: Compiles,
  requires: []
}

data suite: TestSuite = {
  name: "runner_failure",
  claims: [claim_should_compile]
}
"#;
    let dag = compile_clean(source, "test_runner_failure.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].result, ClaimResult::Fail(_)));
}

#[test]
fn test_runner_fails_closed_on_requires_edges() {
    let source = r#"
data claim_with_requires: TestClaim = {
  name: "requires resource",
  source: "let x: Int = 1",
  file_name: "runner_requires.v3",
  predicate: Compiles,
  requires: [{ target: Int }]
}

data suite: TestSuite = {
  name: "runner_requires",
  claims: [claim_with_requires]
}
"#;
    let dag = compile_clean(source, "test_runner_requires.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(matches!(
        &results[0].result,
        ClaimResult::Fail(reason) if reason.contains("resource requirement")
    ));
}

#[test]
fn test_runner_matches_parse_diagnostics_before_dag_exists() {
    let source = r#"
data claim_parse_error: TestClaim = {
  name: "parse diagnostic",
  source: "let x =",
  file_name: "runner_parse_error.v3",
  predicate: FailsWithDiagnostic({ kind: ParseError, detail_contains: AnyDetail }),
  requires: []
}

data suite: TestSuite = {
  name: "runner_parse_error",
  claims: [claim_parse_error]
}
"#;
    let dag = compile_clean(source, "test_runner_parse_error.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].result, ClaimResult::Pass);
}

#[test]
fn test_runner_fails_closed_on_malformed_diagnostic_detail_filter() {
    let source = r#"
data claim_parse_error: TestClaim = {
  name: "parse diagnostic",
  source: "let x =",
  file_name: "runner_malformed_detail.v3",
  predicate: FailsWithDiagnostic({ kind: ParseError, detail_contains: AnyDetail }),
  requires: []
}
"#;
    let dag = compile_clean(source, "test_runner_malformed_detail.dag");
    let mut claim = claim_value(&dag, "claim_parse_error");

    let FieldValue::Variant { payload, .. } = &mut claim.predicate else {
        panic!("predicate should be a variant");
    };
    let [FieldValue::Record(fields)] = payload.as_mut_slice() else {
        panic!("FailsWithDiagnostic should carry a DiagnosticReference record");
    };
    let detail_contains = fields
        .iter_mut()
        .find(|(label, _)| label == "detail_contains")
        .map(|(_, value)| value)
        .expect("DiagnosticReference has detail_contains");
    let FieldValue::Variant { payload, .. } = detail_contains else {
        panic!("detail_contains should be a variant");
    };
    *payload = vec![FieldValue::Literal(LiteralBits::String(
        "not valid for AnyDetail".to_string(),
    ))];

    let result = TestRunner::new(&dag).run_claim(&claim).result;
    assert!(matches!(
        result,
        ClaimResult::Fail(reason) if reason.contains("AnyDetail should not carry payload")
    ));
}

#[test]
fn test_runner_evaluates_cost_bounded_claim() {
    let source = r#"
data claim_cost: TestClaim = {
  name: "cost bounded",
  source: "let x: Int = 0",
  file_name: "runner_cost.v3",
  predicate: CostBounded("x", Eq, 0),
  requires: []
}

data suite: TestSuite = {
  name: "runner_cost",
  claims: [claim_cost]
}
"#;
    let dag = compile_clean(source, "test_runner_cost.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert_all_pass(&results);
}

#[test]
fn test_runner_evaluates_port_has_state_claim() {
    let source = r#"
data claim_state: TestClaim = {
  name: "port has state",
  source: "let x: Int = 0",
  file_name: "runner_port_state.v3",
  predicate: PortHasState("x", Resolved),
  requires: []
}

data suite: TestSuite = {
  name: "runner_port_state",
  claims: [claim_state]
}
"#;
    let dag = compile_clean(source, "test_runner_port_state.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert_all_pass(&results);
}

#[test]
fn test_runner_scopes_bind_lookup_to_claim_source_file() {
    let source = r#"
data claim_state: TestClaim = {
  name: "port state for shadowing bind",
  source: "let sequential: Int = 0",
  file_name: "runner_shadow_bind.v3",
  predicate: PortHasState("sequential", Resolved),
  requires: []
}

data claim_cost: TestClaim = {
  name: "cost for shadowing bind",
  source: "let sequential: Int = 0",
  file_name: "runner_shadow_bind.v3",
  predicate: CostBounded("sequential", Eq, 0),
  requires: []
}

data suite: TestSuite = {
  name: "runner_shadow_bind",
  claims: [claim_state, claim_cost]
}
"#;
    let dag = compile_clean(source, "test_runner_shadow_bind.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 2);
    assert_all_pass(&results);
}

#[test]
fn test_runner_evaluates_output_equals_claim() {
    let source = r#"
data claim_output: TestClaim = {
  name: "output equals",
  source: "data answer: Int = 1",
  file_name: "runner_output.v3",
  predicate: OutputEquals("1"),
  requires: []
}

data suite: TestSuite = {
  name: "runner_output",
  claims: [claim_output]
}
"#;
    let dag = compile_clean(source, "test_runner_output.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert_all_pass(&results);
}

#[test]
fn test_runner_marks_non_day_one_predicates_not_yet_implemented() {
    let source = r#"
data claim_nyi: TestClaim = {
  name: "behavioral observation",
  source: "let x: Int = 0",
  file_name: "runner_nyi.v3",
  // Struct-variant field names are schema metadata today; the surface parser
  // accepts positional payload syntax for authored values.
  predicate: BehavioralObservation(Int, Int, Int),
  requires: []
}

data suite: TestSuite = {
  name: "runner_nyi",
  claims: [claim_nyi]
}
"#;
    let dag = compile_clean(source, "test_runner_nyi.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(matches!(
        &results[0].result,
        ClaimResult::NotYetImplemented(reason)
            if reason.contains("BehavioralObservation")
    ));
}

#[test]
fn mock_backed_invariant_predicate_accepts_declaration_ref_like_lens_output_equals() {
    let source = r#"
data claim: TestClaim = {
  name: "c",
  source: "let _: Int = 0",
  file_name: "f.v3",
  predicate: MockBackedInvariant(Int, Int),
  requires: []
}
"#;
    compile_clean(source, "mock_backed_invariant_harness.v3");
}

#[test]
fn test_runner_dispatches_mock_backed_invariant_claim() {
    let dag = compile_clean(
        MOCK_BACKED_INVARIANT_FIXTURE,
        "src/v3/compiler/tests/fixtures/r1_mock_backed_invariant_gate.dag",
    );
    let results = TestRunner::new(&dag).run_suite("mock_backed_invariant_suite");

    assert_eq!(results.len(), 1);
    assert_all_pass(&results);
}

#[test]
fn test_runner_mock_backed_invariant_fails_when_subject_source_fails() {
    let source = r#"
data subject_ref: Int = 0
data invariant_ref: Int = 0

data claim: TestClaim = {
  name: "bad mock subject",
  source: "let x: Bool = 1",
  file_name: "bad_mock_subject.v3",
  predicate: MockBackedInvariant(subject_ref, invariant_ref),
  requires: []
}

data suite: TestSuite = {
  name: "bad_mock_subject_suite",
  claims: [claim]
}
"#;
    let dag = compile_clean(source, "bad_mock_subject_harness.v3");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(matches!(
        &results[0].result,
        ClaimResult::Fail(reason)
            if reason.contains("subject_ref") && reason.contains("invariant_ref")
    ));
}

#[test]
fn test_runner_runs_sub_match_over_user_sum_gate() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gate = manifest_dir.join("tests/fixtures/r1_gates.dag");
    let source =
        std::fs::read_to_string(&gate).unwrap_or_else(|err| panic!("read {gate:?}: {err}"));
    let dag = compile_clean(&source, "src/v3/compiler/tests/fixtures/r1_gates.dag");
    let results = TestRunner::new(&dag).run_suite("sub_match_over_user_sum_gate");

    assert_all_pass(&results);
}
