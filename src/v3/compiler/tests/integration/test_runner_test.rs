//! **Layer:** integration

use std::path::PathBuf;

use v3_compiler::dag::{FieldValue, LiteralBits};
use v3_compiler::diagnostics::Diagnostic;
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

#[test]
fn r1_merge_sort_pair_fixture_cost_is_hit_three() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("tests/fixtures/r1_merge_sort_pair.v3");
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let dag = compile_clean(&source, "r1_merge_sort_pair.v3");
    let bind = dag.nodes().iter().find_map(|n| match n {
        v3_compiler::dag::Behavior::Bind(b) if b.name == "merge_sort_out" => Some(b.clone()),
        _ => None,
    });
    let Some(bind) = bind else {
        panic!("merge_sort_out bind missing");
    };
    let cost = v3_compiler::lens_cost::cost_of(&dag, &bind.value);
    assert_eq!(cost, v3_compiler::lens_cost::CostLookup::Hit(3));
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
fn test_runner_data_bodies_reject_requires_empty_call_today() {
    // Checklist item (2) in `docs/briefs/r1-testgen-manager.md` — `requires: []`
    // vs `requires: empty()`: in a `data` body, only the `[]` list literal lowers
    // today. `empty()` trips the M1(2.8) class-5 gap (the data body cannot be
    // structurally validated), so the runner path on
    // `src/v3/compiler/tests/dag/*.dag` **must** stay with `[]`. `empty()` remains
    // valid in `let` bindings (Brief D `.v3` fixtures under
    // `tests/fixtures/t_pb_b_brief_d/` use it), but those bindings are not
    // `Declaration`s and so are not directly `run_suite`-consumable.
    //
    // This is the standing receipt: if the M1(2.8) class-5 restriction lifts and
    // `data` bodies start accepting `empty()`, this test flips to a green
    // "equivalence" test — at which point the `.dag` files can optionally migrate
    // off the `[]` literal.
    let source = r#"
import std.list { empty }

data claim_empty_requires: TestClaim = {
  name: "empty() requires compiles",
  source: "let x: Int = 1",
  file_name: "runner_empty_requires.v3",
  predicate: Compiles,
  requires: empty()
}
"#;
    match compile_to_dag(source, "test_runner_empty_requires.dag") {
        Err(CompileError::Semantic(dag)) => {
            // Variant-level check: the class-5 gap surfaces as a `ResolveError`
            // on the `requires` field of `claim_empty_requires`. Binding to the
            // variant + `name` payload is sturdier than a raw message-substring
            // check — the message may be rephrased, but the variant + the
            // data-decl name are the stable facts. Message substring is retained
            // as the secondary signal until a diagnostic-code vocabulary lands
            // (cross-link: review observation on #736).
            let found_resolve_error = dag.diagnostics().iter().any(|(_, diag)| {
                matches!(
                    diag,
                    Diagnostic::ResolveError { name, .. }
                        if name.contains("claim_empty_requires")
                            && name.contains("requires")
                            && name.contains("list literal")
                )
            });
            assert!(
                found_resolve_error,
                "expected ResolveError naming `claim_empty_requires` / `requires` / `list literal`, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
        }
        other => {
            panic!("expected `requires: empty()` to be rejected by M1(2.8) today, got {other:?}")
        }
    }
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
fn test_runner_evaluates_execute_command_pass_and_fail() {
    let pass = r#"
data claim_true: TestClaim = {
  name: "true exits 0",
  source: "let x: Int = 0",
  file_name: "runner_exec_pass.v3",
  predicate: ExecuteCommand("true", [], 0),
  requires: []
}
data suite_pass: TestSuite = { name: "execute_command_pass", claims: [claim_true] }
"#;
    let dag = compile_clean(pass, "test_runner_exec_pass.dag");
    let results = TestRunner::new(&dag).run_suite("suite_pass");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].result, ClaimResult::Pass);

    let mismatch = r#"
data claim_mismatch: TestClaim = {
  name: "expect 1, true exits 0",
  source: "let x: Int = 0",
  file_name: "runner_exec_mismatch.v3",
  predicate: ExecuteCommand("true", [], 1),
  requires: []
}
data suite_mismatch: TestSuite = { name: "execute_command_mismatch", claims: [claim_mismatch] }
"#;
    let dag = compile_clean(mismatch, "test_runner_exec_mismatch.dag");
    let results = TestRunner::new(&dag).run_suite("suite_mismatch");
    assert_eq!(results.len(), 1);
    let ClaimResult::Fail(msg) = &results[0].result else {
        panic!(
            "expected Fail on exit mismatch, got {:?}",
            results[0].result
        );
    };
    assert!(
        msg.contains("exit code mismatch") && msg.contains("expected 1") && msg.contains("got 0"),
        "unexpected fail message: {msg}"
    );
}

/// Missing-host-binary triage: on some hosts the failure is a **spawn** `Err` from
/// `std::process::Command`; on **Linux** the runner usually prefixes with `unshare(1)`,
/// so `unshare` may spawn successfully while the inner `exec` fails, surfacing as **exit
/// code mismatch** (e.g. 127) instead; or `unshare` itself may fail to start. All remain
/// typed `Fail` and distinguishable from other ExecuteCommand outcomes (PR #792 / claude review).
#[test]
fn test_runner_execute_command_missing_binary_is_distinguishable_fail() {
    let source = r#"
data claim: TestClaim = {
  name: "no such binary",
  source: "let x: Int = 0",
  file_name: "runner_exec_spawn.v3",
  predicate: ExecuteCommand(
    "no_such_v3_test_binary_a7f2c1",
    [],
    0
  ),
  requires: []
}
data suite: TestSuite = { name: "execute_command_spawn", claims: [claim] }
"#;
    let dag = compile_clean(source, "test_runner_exec_spawn.dag");
    let result = &TestRunner::new(&dag).run_suite("suite")[0].result;
    let ClaimResult::Fail(msg) = result else {
        panic!("expected Fail, got {result:?}");
    };
    // After the typed-outcome refactor + helper-binary wiring (PR #1049 / Worker 4),
    // the only production-emittable phrasings for missing-binary on this seam are:
    //   * `SpawnFailed { wrapper: None }`         → "ExecuteCommand spawn error: ..."
    //   * `SpawnFailed { wrapper: Some(_) }`      → "wrapper failed to spawn"
    //   * `SpawnFailed` from helper-probe miss    → "not executable"
    //   * `SetupFailed { NamespaceSetupAndDirectSpawnFailed }`
    //                                             → "namespace setup failed"
    //   * `SetupFailed { HelperBinaryMissing }`
    //                                             → "helper not found"
    //   * `Mismatch`                              → "exit code mismatch"
    assert!(
        msg.contains("spawn error")
            || msg.contains("exit code mismatch")
            || msg.contains("not executable")
            || msg.contains("namespace setup failed")
            || msg.contains("helper not found")
            || (msg.contains("unshare(1)") && msg.contains("wrapper failed to spawn")),
        "expected missing-binary or unshare triage; got: {msg}"
    );
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
fn test_runner_dispatches_pb_census_predicate_shapes() {
    let source = r#"
import std.verification {
  compiler_std_positive_set_ratchet,
  expected_hand_authored_non_test,
  lens_producer_files_subset_predicate
}

data census_authority: Int = 0

data census_bound_claim: TestClaim = {
  name: "pb_hand_rust_at_shim_floor",
  source: "let x: Int = 1",
  file_name: "pb_hand_rust_at_shim_floor.v3",
  predicate: CensusBoundCheck(census_authority, expected_hand_authored_non_test, 0),
  requires: []
}

data census_subset_claim: TestClaim = {
  name: "lens_producer_files_remaining",
  source: "let x: Int = 1",
  file_name: "lens_producer_files_remaining.v3",
  predicate: CensusSubsetCount {
    authority: census_authority,
    list_constant: expected_hand_authored_non_test,
    subset_predicate: lens_producer_files_subset_predicate
  },
  requires: []
}

data fixed_point_claim: TestClaim = {
  name: "pb_self_compile_fixed_point",
  source: "let x: Int = 1",
  file_name: "pb_self_compile_fixed_point.v3",
  predicate: FixedPointConverges("default_fixed_point_source", "pipeline_stage_snapshots"),
  requires: []
}

data ratchet_zero_claim: TestClaim = {
  name: "pb_compiler_std_ratchet_zero",
  source: "let x: Int = 1",
  file_name: "pb_compiler_std_ratchet_zero.v3",
  predicate: RatchetZero {
    authority: census_authority,
    ratchet_kind: compiler_std_positive_set_ratchet
  },
  requires: []
}

data generated_from_dag_claim: TestClaim = {
  name: "pb_test_file_generated_from_dag",
  source: "let x: Int = 1",
  file_name: "pb_test_file_generated_from_dag.v3",
  predicate: GeneratedFromDag(census_authority, ["src/v3/compiler/tests/integration.rs"]),
  requires: []
}

data suite: TestSuite = {
  name: "pb_census_predicate_shapes",
  claims: [
    census_bound_claim,
    census_subset_claim,
    fixed_point_claim,
    ratchet_zero_claim,
    generated_from_dag_claim
  ]
}
"#;
    let dag = compile_clean(source, "pb_census_predicate_shapes.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 5);
    let result_for = |claim_name: &str| {
        &results
            .iter()
            .find(|result| result.claim_name == claim_name)
            .unwrap_or_else(|| panic!("missing claim result for `{claim_name}`"))
            .result
    };
    let assert_fail_contains = |claim_name: &str, expected: &str| {
        assert!(
            matches!(result_for(claim_name), ClaimResult::Fail(reason) if reason.contains(expected)),
            "expected `{claim_name}` to fail with `{expected}`, got {:?}",
            result_for(claim_name)
        );
    };
    assert_fail_contains(
        "pb_hand_rust_at_shim_floor",
        "expected_hand_authored_non_test",
    );
    assert_fail_contains(
        "lens_producer_files_remaining",
        "lens-producer subset observed",
    );
    assert_eq!(
        result_for("pb_self_compile_fixed_point"),
        &ClaimResult::Pass
    );
    assert_fail_contains(
        "pb_compiler_std_ratchet_zero",
        "compiler_std_positive_set_ratchet",
    );
    assert_fail_contains(
        "pb_test_file_generated_from_dag",
        "not in the generated-file authority",
    );
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
    assert_eq!(
        results[0].claim_name,
        "testgen_mock_backed_integration_safe"
    );
    assert_eq!(results[0].result, ClaimResult::Pass);
}

#[test]
fn test_runner_mock_backed_invariant_fails_when_invariant_rejects_subject_output() {
    let source = r#"
data subject_ref: Int = 0
data invariant_ref: Int = 0

data claim: TestClaim = {
  name: "mock backed clean source",
  source: "fn subject_ref() -> Int = 500\n\nfn invariant_ref(code: Int) -> Bool = code == 200\n\nlet _: Int = 0\n",
  file_name: "clean_mock_subject.v3",
  predicate: MockBackedInvariant(subject_ref, invariant_ref),
  requires: []
}

data suite: TestSuite = {
  name: "clean_mock_subject_suite",
  claims: [claim]
}
"#;
    let dag = compile_clean(source, "clean_mock_subject_harness.v3");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(matches!(
        &results[0].result,
        ClaimResult::Fail(reason)
            if reason.contains("invariant") && reason.contains("Bool(true)")
    ));
}

#[test]
fn lens_output_equals_predicate_accepts_declaration_ref_literals_like_mock_invariant() {
    let source = r#"
data claim: TestClaim = {
  name: "c",
  source: "let _: Int = 0",
  file_name: "f.v3",
  predicate: LensOutputEquals(Int, Int, Int),
  requires: []
}

data suite: TestSuite = {
  name: "s",
  claims: [claim]
}
"#;
    compile_clean(source, "lens_output_equals_int_harness.v3");
}

#[test]
fn binary_dimension_report_equals_rejects_non_report_refs() {
    let source = r#"
module test.binary_dimension_report_equals_non_report

import std.verification { BinaryDimensionReportEquals, TestClaim, TestSuite }

fn left_report() -> Int = 0
fn right_report() -> Int = 0

data claim: TestClaim = {
  name: "non-report refs fail closed",
  source: "let _: Int = 0",
  file_name: "binary_dimension_report_non_report.v3",
  predicate: BinaryDimensionReportEquals(left_report, right_report),
  requires: []
}

data suite: TestSuite = {
  name: "s",
  claims: [claim]
}
"#;
    let dag = compile_clean(source, "binary_dimension_report_non_report_harness.v3");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(
        matches!(
            &results[0].result,
            ClaimResult::Fail(reason)
                if reason.contains("BinaryDimensionReportEquals")
                    && reason.contains("DimensionReport<C>")
                    && reason.contains("left_report")
        ),
        "expected non-report refs to fail closed, got {:?}",
        results[0].result
    );
}

#[test]
fn binary_dimension_report_equals_rejects_mismatched_report_carriers() {
    let source = r#"
module test.binary_dimension_report_equals_mismatched_carriers

import std.dimensions { DimensionReport }
import std.verification { BinaryDimensionReportEquals, TestClaim, TestSuite }

type left_report = DimensionReport<Int>
type right_report = DimensionReport<String>

data claim: TestClaim = {
  name: "mismatched report carriers fail closed",
  source: "let _: Int = 0",
  file_name: "binary_dimension_report_mismatched_carriers.v3",
  predicate: BinaryDimensionReportEquals(left_report, right_report),
  requires: []
}

data suite: TestSuite = {
  name: "s",
  claims: [claim]
}
"#;
    let dag = compile_clean(
        source,
        "binary_dimension_report_mismatched_carriers_harness.v3",
    );
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(
        matches!(
            &results[0].result,
            ClaimResult::Fail(reason)
                if reason.contains("BinaryDimensionReportEquals")
                    && reason.contains("same carrier C")
                    && reason.contains("Int")
                    && reason.contains("String")
        ),
        "expected mismatched report carriers to fail closed, got {:?}",
        results[0].result
    );
}

#[test]
fn binary_dimension_report_equals_accepts_same_report_carrier_shape() {
    let source = r#"
module test.binary_dimension_report_equals_same_carrier

import std.dimensions { DimensionReport }
import std.verification { BinaryDimensionReportEquals, TestClaim, TestSuite }

type left_report = DimensionReport<Int>
type right_report = DimensionReport<Int>

data claim: TestClaim = {
  name: "same report carrier passes shape validation",
  source: "let _: Int = 0",
  file_name: "binary_dimension_report_same_carrier.v3",
  predicate: BinaryDimensionReportEquals(left_report, right_report),
  requires: []
}

data suite: TestSuite = {
  name: "s",
  claims: [claim]
}
"#;
    let dag = compile_clean(source, "binary_dimension_report_same_carrier_harness.v3");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(
        matches!(
            &results[0].result,
            ClaimResult::NotYetImplemented(reason)
                if reason.contains("BinaryDimensionReportEquals")
                    && reason.contains("structural shape is valid")
        ),
        "expected same report carrier shape to reach NYI evaluation path, got {:?}",
        results[0].result
    );
}

#[test]
fn binary_dimension_report_equals_accepts_structurally_equivalent_report_carriers() {
    let source = r#"
module test.binary_dimension_report_equals_structural_carriers

import std.dimensions { DimensionReport }
import std.verification { BinaryDimensionReportEquals, TestClaim, TestSuite }

type LeftCarrier { value: Int }
type RightCarrier { value: Int }

type left_report = DimensionReport<LeftCarrier>
type right_report = DimensionReport<RightCarrier>

data claim: TestClaim = {
  name: "structurally equivalent report carriers pass shape validation",
  source: "let _: Int = 0",
  file_name: "binary_dimension_report_structural_carriers.v3",
  predicate: BinaryDimensionReportEquals(left_report, right_report),
  requires: []
}

data suite: TestSuite = {
  name: "s",
  claims: [claim]
}
"#;
    let dag = compile_clean(
        source,
        "binary_dimension_report_structural_carriers_harness.v3",
    );
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(
        matches!(
            &results[0].result,
            ClaimResult::NotYetImplemented(reason)
                if reason.contains("BinaryDimensionReportEquals")
                    && reason.contains("structural shape is valid")
        ),
        "expected structurally equivalent report carriers to pass shape validation, got {:?}",
        results[0].result
    );
}

#[test]
fn lens_output_equals_malformed_claim_source_fails_closed_with_literal_input() {
    // INVARIANTS P3 / TESTING: tokenize/parse failure for `TestClaim.source` must surface as
    // `Fail`, not fall back to the fixture lens (which could still `Pass` on a stub).
    let source = r#"
module test.lens_output_equals_bad_claim_source

import std.substrate { Dag }
import std.verification { LensOutputEquals, TestClaim, TestSuite }

fn zero_on_dag(d: Dag) -> Int = 0

data lens_input: Int = 7

data lens_expected: Int = 0

data claim_bad_program: TestClaim = {
  name: "bad source should not fall back to fixture lens",
  source: "this is not valid v3 >>>",
  file_name: "lens_output_equals_bad_claim_source.v3",
  predicate: LensOutputEquals(zero_on_dag, lens_input, lens_expected),
  requires: []
}

data suite: TestSuite = {
  name: "suite",
  claims: [claim_bad_program]
}
"#;
    let dag = compile_clean(source, "lens_output_equals_bad_claim_source_harness.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(
        matches!(
            &results[0].result,
            ClaimResult::Fail(reason)
                if reason.contains("did not compile")
                    || reason.contains("failed inference")
        ),
        "expected compile-time failure for malformed claim.source, got {:?}",
        results[0].result
    );
}

#[test]
fn r1_canonical_complexity_lens_bytes_include_cost_of() {
    let bytes = v3_compiler::test_runner::R1_CANONICAL_COMPLEXITY_LENS;
    assert!(
        bytes.contains("fn complexity_of"),
        "canonical lens should declare complexity_of"
    );
    assert!(
        bytes.contains("fn compute_summaries") && bytes.contains("fn seed_bind_params"),
        "canonical `complexity.dag` bytes should include the forward-fold spine, not just the `complexity_of` signature"
    );
}

#[test]
fn test_runner_dispatches_r1_gates_lens_output_equals_claim() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gate = manifest_dir.join("tests/fixtures/r1_gates.dag");
    let source =
        std::fs::read_to_string(&gate).unwrap_or_else(|err| panic!("read {gate:?}: {err}"));
    let dag = compile_clean(&source, "src/v3/compiler/tests/fixtures/r1_gates.dag");
    let results = TestRunner::new(&dag).run_suite("r1_lens_output_equals_suite");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].claim_name, "lens_output_equals_gate");
    assert_eq!(results[0].result, ClaimResult::Pass);
}

/// Retired string-keyed program-input bridge: only `ProgramInput {}` selects reflection of
/// `TestClaim.source`. The historical declaration *name* on a non-carrier type must not spoof
/// the structural path (B4.1 residual / brief addendum).
#[test]
fn lens_output_equals_retired_sentinel_name_on_int_is_not_program_input() {
    let source = r#"
module test.lens_output_equals_sentinel_name_spoof

import std.list { fold }
import std.substrate { Dag, Behavior }
import std.types { Int }
import std.verification { LensOutputEquals, TestClaim, TestSuite }

fn count_named_bind(behavior: Behavior) -> Int =
  match behavior {
    Value(v) => 0
    Transform(t) => 0
    Branch(b) => 0
    Loop(l) => 0
    Bind(bind) => if bind.name == "" then 0 else 1
  }

fn named_function_count(d: Dag) -> Int =
  fold(d.nodes, 0, |acc, behavior| acc + count_named_bind(behavior))

// Name collides with the pre–B4.1 string sentinel only; type is ordinary `Int`, not `ProgramInput`.
data r1_lens_output_input_from_program: Int = 0

data spoof_expected: Int = 1

data spoof_claim: TestClaim = {
  name: "sentinel_name_spoof",
  source: "let x: Int = 1\n",
  file_name: "lens_output_equals_sentinel_spoof.v3",
  predicate: LensOutputEquals(
    named_function_count,
    r1_lens_output_input_from_program,
    spoof_expected
  ),
  requires: []
}

data spoof_suite: TestSuite = {
  name: "spoof_suite",
  claims: [spoof_claim]
}
"#;
    let dag = compile_clean(source, "lens_output_equals_sentinel_spoof_harness.dag");
    let results = TestRunner::new(&dag).run_suite("spoof_suite");
    assert_eq!(results.len(), 1);
    assert_ne!(
        results[0].result,
        ClaimResult::Pass,
        "expected Fail: name-only collision must not select claim-program reflection (structural `ProgramInput` only)"
    );
}

#[test]
fn test_runner_algebraic_law_commutativity_passes_for_int_add() {
    let source = r#"
module test.algebraic_law_commutativity_nyi

import std.verification { AlgebraicLaw, TestClaim, TestSuite }

fn lens_placeholder(a: Int, b: Int) -> Int = a + b

data claim_comm: TestClaim = {
  name: "algebraic law commutativity",
  source: "fn lens_placeholder(a: Int, b: Int) -> Int = a + b",
  file_name: "algebraic_law_comm.v3",
  predicate: AlgebraicLaw(Commutativity, lens_placeholder),
  requires: []
}

data suite: TestSuite = {
  name: "algebraic_law_commutativity_suite",
  claims: [claim_comm]
}
"#;
    let dag = compile_clean(source, "test_runner_algebraic_law_comm.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].result, ClaimResult::Pass);
}

#[test]
fn test_runner_algebraic_law_commutativity_fails_for_subtraction() {
    let source = r#"
module test.algebraic_law_commutativity_fail

import std.verification { AlgebraicLaw, TestClaim, TestSuite }

fn lens_placeholder(a: Int, b: Int) -> Int = a - b

data claim_comm: TestClaim = {
  name: "algebraic law commutativity fails",
  source: "fn lens_placeholder(a: Int, b: Int) -> Int = a - b",
  file_name: "algebraic_law_comm_fail.v3",
  predicate: AlgebraicLaw(Commutativity, lens_placeholder),
  requires: []
}

data suite: TestSuite = {
  name: "algebraic_law_commutativity_fail_suite",
  claims: [claim_comm]
}
"#;
    let dag = compile_clean(source, "test_runner_algebraic_law_comm_fail.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(matches!(&results[0].result, ClaimResult::Fail(_)));
}

#[test]
fn test_runner_algebraic_law_identity_names_identity_edge_blocker() {
    let source = r#"
module test.algebraic_law_identity_nyi

import std.verification { AlgebraicLaw, TestClaim, TestSuite }

fn lens_placeholder(a: Int, b: Int) -> Int = a + b

data claim_identity: TestClaim = {
  name: "algebraic law identity",
  source: "fn lens_placeholder(a: Int, b: Int) -> Int = a + b",
  file_name: "algebraic_law_identity.v3",
  predicate: AlgebraicLaw(Identity, lens_placeholder),
  requires: []
}

data suite: TestSuite = {
  name: "algebraic_law_identity_suite",
  claims: [claim_identity]
}
"#;
    let dag = compile_clean(source, "test_runner_algebraic_law_identity.dag");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(matches!(
        &results[0].result,
        ClaimResult::NotYetImplemented(_)
    ));
}

#[test]
fn test_runner_runs_r1_lane_e_suite() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gate = manifest_dir.join("tests/fixtures/r1_gates.dag");
    let source =
        std::fs::read_to_string(&gate).unwrap_or_else(|err| panic!("read {gate:?}: {err}"));
    let dag = compile_clean(&source, "src/v3/compiler/tests/fixtures/r1_gates.dag");
    let results = TestRunner::new(&dag).run_suite("r1_lane_e_suite");
    assert_eq!(results.len(), 3);
    assert_all_pass(&results);
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

#[test]
fn test_runner_runs_sub_type_alias_where_lowers_gate() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gate = manifest_dir.join("tests/fixtures/r1_gates.dag");
    let source =
        std::fs::read_to_string(&gate).unwrap_or_else(|err| panic!("read {gate:?}: {err}"));
    let dag = compile_clean(&source, "src/v3/compiler/tests/fixtures/r1_gates.dag");
    let results = TestRunner::new(&dag).run_suite("sub_type_alias_where_lowers_gate");

    assert_all_pass(&results);
}

#[test]
fn test_runner_runs_p0_repeat_string_correct_gate() {
    crate::common::assert_p0_repeat_string_correct_gate_passes();
}

#[test]
fn test_runner_runs_p0_host_sentinel_and_rest_gate() {
    crate::common::assert_p0_host_sentinel_and_rest_gate_passes();
}
