//! **Layer:** integration

use std::path::PathBuf;

use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::{compile_to_dag, CompileError};

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
  name: "mock backed invariant",
  source: "let x: Int = 0",
  file_name: "runner_nyi.v3",
  predicate: MockBackedInvariant(Int, Bool),
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
    assert_eq!(results[0].result, ClaimResult::NotYetImplemented);
}

#[test]
#[ignore = "Brief 1 dependency: r1_gates.dag/user_authored_lens_compiles_gate is not present in this worktree yet"]
fn test_runner_runs_user_authored_lens_compiles_gate() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repo root")
        .to_path_buf();
    let gate = repo_root.join("src/v3/r1_gates.dag");
    let source =
        std::fs::read_to_string(&gate).unwrap_or_else(|err| panic!("read {gate:?}: {err}"));
    let dag = compile_clean(&source, "src/v3/r1_gates.dag");
    let results = TestRunner::new(&dag).run_suite("user_authored_lens_compiles_gate");

    assert_all_pass(&results);
}
