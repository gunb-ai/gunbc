use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    Behavior, Dag, DeclarationId, FieldValue, LiteralBits, PortState, TypeConnective, ValueBody,
};
use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

fn compile_any(src: &str, file: &str) -> Dag {
    match compile_to_dag(src, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}

fn find_named(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("declaration `{name}` not found"))
        .id
}

fn record_fields(dag: &Dag, name: &str) -> Vec<String> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Conj { children } => {
            children.iter().map(|field| field.label.clone()).collect()
        }
        other => panic!("expected `{name}` to lower to a Conj, got {other:?}"),
    }
}

fn sum_variants(dag: &Dag, name: &str) -> Vec<(String, Vec<String>)> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .map(|variant| {
                let payload = match &dag.declaration(variant.ty).connective {
                    TypeConnective::Conj { children } => {
                        children.iter().map(|field| field.label.clone()).collect()
                    }
                    other => panic!(
                        "expected variant `{}` under `{name}` to lower to a Conj payload, got {other:?}",
                        variant.label
                    ),
                };
                (variant.label.clone(), payload)
            })
            .collect(),
        other => panic!("expected `{name}` to lower to a Disj, got {other:?}"),
    }
}

fn bind_value_type_decl(dag: &Dag, name: &str) -> DeclarationId {
    let value_port = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            Behavior::Bind(bind) if bind.name == name => Some(bind.value),
            _ => None,
        })
        .unwrap_or_else(|| panic!("bind `{name}` not found"));
    match dag.port(value_port).state() {
        PortState::Resolved(ty) => ty.declaration,
        other => panic!("bind `{name}` did not resolve, got {other:?}"),
    }
}

#[test]
fn bootstrap_loads_verification_authority_types() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load staged std.verification cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    assert_eq!(
        record_fields(&dag, "TestClaim"),
        vec!["name", "source", "file_name", "predicate", "requires"]
    );
    assert_eq!(record_fields(&dag, "TestSuite"), vec!["name", "claims"]);
    assert_eq!(
        sum_variants(&dag, "DiagnosticKind"),
        vec![
            (String::from("TokenizerError"), Vec::new()),
            (String::from("ParseError"), Vec::new()),
            (String::from("TypeMismatch"), Vec::new()),
            (String::from("ArityMismatch"), Vec::new()),
            (String::from("ResolveError"), Vec::new()),
            (String::from("UnitMismatch"), Vec::new()),
            (String::from("BranchConditionNotBool"), Vec::new()),
            (String::from("MagnitudeOutOfRange"), Vec::new()),
            (String::from("MalformedIntegerRangeFact"), Vec::new()),
            (String::from("NominalOpacityViolation"), Vec::new()),
        ]
    );
    assert_eq!(
        record_fields(&dag, "DiagnosticReference"),
        vec![String::from("kind"), String::from("detail_contains")]
    );
    assert_eq!(
        sum_variants(&dag, "DiagnosticDetailExpectation"),
        vec![
            (String::from("AnyDetail"), Vec::new()),
            (String::from("Contains"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "PortStateExpectation"),
        vec![
            (String::from("Resolved"), Vec::new()),
            (String::from("Unresolved"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "AlgebraicLawKind"),
        vec![
            (String::from("Associativity"), Vec::new()),
            (String::from("Commutativity"), Vec::new()),
            (String::from("Identity"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "TestPredicate"),
        vec![
            (String::from("Compiles"), Vec::new()),
            (
                String::from("FailsWithDiagnostic"),
                vec![String::from("_0")],
            ),
            (String::from("OutputEquals"), vec![String::from("expected")],),
            (
                String::from("PortHasState"),
                vec![String::from("bind_name"), String::from("state")],
            ),
            (
                String::from("DeclarationHasRefinement"),
                vec![String::from("declaration_name")],
            ),
            (
                String::from("CostBounded"),
                vec![
                    String::from("bind_name"),
                    String::from("comparator"),
                    String::from("bound"),
                ],
            ),
            (
                String::from("BehavioralObservation"),
                vec![
                    String::from("subject"),
                    String::from("input_sample"),
                    String::from("expected_output"),
                ],
            ),
            (
                String::from("MockBackedInvariant"),
                vec![String::from("subject"), String::from("invariant")],
            ),
            (
                String::from("ExecuteCommand"),
                vec![
                    String::from("command"),
                    String::from("args"),
                    String::from("expect_exit_code"),
                ],
            ),
            (
                String::from("ForAllTargets"),
                vec![
                    String::from("command"),
                    String::from("args"),
                    String::from("expect_exit_code"),
                ],
            ),
            (
                String::from("LensOutputEquals"),
                vec![
                    String::from("lens_ref"),
                    String::from("input_ref"),
                    String::from("expected_ref"),
                ],
            ),
            (
                String::from("DifferentialEquals"),
                vec![
                    String::from("subject_ref"),
                    String::from("oracle_ref"),
                    String::from("input_ref"),
                ],
            ),
            (
                String::from("SymbolicCostExprEquals"),
                vec![String::from("expected")],
            ),
            (
                String::from("PerfWithinBaseline"),
                vec![
                    String::from("subject"),
                    String::from("comparator"),
                    String::from("baseline_ref"),
                ],
            ),
            (
                String::from("BinaryDimensionReportEquals"),
                vec![
                    String::from("left_report_ref"),
                    String::from("right_report_ref"),
                ],
            ),
            (
                String::from("AlgebraicLaw"),
                vec![String::from("law"), String::from("lens_ref")],
            ),
            (
                String::from("CensusBoundCheck"),
                vec![
                    String::from("authority"),
                    String::from("list_constant"),
                    String::from("bound"),
                ],
            ),
            (
                String::from("CensusSubsetCount"),
                vec![
                    String::from("authority"),
                    String::from("list_constant"),
                    String::from("subset_predicate"),
                ],
            ),
            (
                String::from("FixedPointConverges"),
                vec![String::from("compile_target"), String::from("expected")],
            ),
            (
                String::from("RatchetZero"),
                vec![String::from("authority"), String::from("ratchet_kind")],
            ),
            (
                String::from("GeneratedFromDag"),
                vec![String::from("authority"), String::from("generated_paths")],
            ),
            (
                String::from("ReleaseDeferredClaim"),
                vec![
                    String::from("deferred_gate"),
                    String::from("target_lane"),
                    String::from("authority_doc"),
                ],
            ),
            (
                String::from("SubstrateResearchDeferredClaim"),
                vec![
                    String::from("deferred_gate"),
                    String::from("target_lane"),
                    String::from("authority_doc"),
                ],
            ),
            (
                String::from("BridgeLedgerZero"),
                vec![String::from("ledger")],
            ),
        ]
    );
}

#[test]
fn verification_predicate_witnesses_compile_cleanly() {
    let src = r#"
import std.list { empty }

let pred_compiles: TestPredicate = Compiles
let pred_fails: TestPredicate = FailsWithDiagnostic({ kind: ResolveError, detail_contains: Contains("missing") })
let pred_fails_kind: TestPredicate = FailsWithDiagnostic({ kind: TypeMismatch, detail_contains: AnyDetail })
let pred_output: TestPredicate = OutputEquals("let x: Int = 1")
let pred_port_resolved: TestPredicate = PortHasState("answer", Resolved)
let pred_port_unresolved: TestPredicate = PortHasState("missing", Unresolved)
let pred_decl_refine: TestPredicate = DeclarationHasRefinement("PositiveInt")
let pred_cost_eq: TestPredicate = CostBounded("answer", Eq, 8)
let pred_cost_above: TestPredicate = CostBounded("answer", Gt, 3)
let pred_exec: TestPredicate = ExecuteCommand("true", empty(), 0)
let pred_all_targets: TestPredicate = ForAllTargets("true", empty(), 0)

let claim_compiles: TestClaim = {
  name: "compiles",
  source: "let x: Int = 1",
  file_name: "compiles.v3",
  predicate: pred_compiles,
  requires: empty()
}

let claim_fails: TestClaim = {
  name: "fails",
  source: "let x: Bool = 1",
  file_name: "fails.v3",
  predicate: pred_fails,
  requires: empty()
}

let claim_alias_refine: TestClaim = {
  name: "alias_where_refine",
  source: "type PositiveInt = Int where PositiveInt > 0",
  file_name: "alias_refine.v3",
  predicate: pred_decl_refine,
  requires: empty()
}

let suite: TestSuite = {
  name: "verification_smoke",
  claims: [claim_compiles, claim_fails, claim_alias_refine]
}
"#;

    let dag = compile_any(src, "verification_witnesses.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "verification witnesses should compile cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let test_predicate = find_named(&dag, "TestPredicate");
    for bind in [
        "pred_compiles",
        "pred_fails",
        "pred_fails_kind",
        "pred_output",
        "pred_port_resolved",
        "pred_port_unresolved",
        "pred_decl_refine",
        "pred_cost_eq",
        "pred_cost_above",
        "pred_exec",
        "pred_all_targets",
    ] {
        assert_eq!(bind_value_type_decl(&dag, bind), test_predicate);
    }
    assert_eq!(
        bind_value_type_decl(&dag, "claim_compiles"),
        find_named(&dag, "TestClaim")
    );
    assert_eq!(
        bind_value_type_decl(&dag, "claim_fails"),
        find_named(&dag, "TestClaim")
    );
    assert_eq!(
        bind_value_type_decl(&dag, "claim_alias_refine"),
        find_named(&dag, "TestClaim")
    );
    assert_eq!(
        bind_value_type_decl(&dag, "suite"),
        find_named(&dag, "TestSuite")
    );
}

/// PR-D (Evaluator Manager): cross-target equivalence harness — slice 0 `Compiles` claim plus
/// slice 1 `DifferentialEquals` scaffold both compile and pass under `TestRunner` (see
/// `docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`).
const R2_PR_D_HARNESS_FIXTURE: &str =
    include_str!("../fixtures/r2_evaluator_cross_target_equivalence_harness_primitives.dag");
const R2_PR_D_HARNESS_FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/r2_evaluator_cross_target_equivalence_harness_primitives.dag";

#[test]
fn r2_pr_d_cross_target_equivalence_harness_primitives_suite_passes() {
    let dag = match compile_to_dag(R2_PR_D_HARNESS_FIXTURE, R2_PR_D_HARNESS_FIXTURE_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{R2_PR_D_HARNESS_FIXTURE_PATH}: expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{R2_PR_D_HARNESS_FIXTURE_PATH} should lower without module diagnostics. Got `Err(Semantic)`: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("unexpected compile error for {R2_PR_D_HARNESS_FIXTURE_PATH}: {other:?}"),
    };

    let results = TestRunner::new(&dag)
        .run_suite("r2_evaluator_cross_target_equivalence_harness_primitives_suite");
    assert_eq!(results.len(), 2);
    assert_eq!(
        results[0].claim_name,
        "evaluator_cross_target_equivalence_harness_primitives_landed"
    );
    assert_eq!(results[0].result, ClaimResult::Pass);
    assert_eq!(
        results[1].claim_name,
        "evaluator_cross_target_equivalence_harness_primitives_differential_scaffold"
    );
    assert_eq!(results[1].result, ClaimResult::Pass);
}

/// PR-A (Evaluator Manager): runtime-value model — named structural gate
/// `evaluator_runtime_value_model_landed` compiles and passes `Compiles`
/// under `TestRunner` (see `docs/briefs/r2-pr-a-runtime-value-model.md`).
const R2_PR_A_RUNTIME_VALUE_FIXTURE: &str =
    include_str!("../fixtures/r2_evaluator_runtime_value_model.dag");
const R2_PR_A_RUNTIME_VALUE_FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/r2_evaluator_runtime_value_model.dag";

#[test]
fn r2_pr_a_runtime_value_model_suite_passes() {
    let dag = match compile_to_dag(
        R2_PR_A_RUNTIME_VALUE_FIXTURE,
        R2_PR_A_RUNTIME_VALUE_FIXTURE_PATH,
    ) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{R2_PR_A_RUNTIME_VALUE_FIXTURE_PATH}: expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{R2_PR_A_RUNTIME_VALUE_FIXTURE_PATH} should lower without module diagnostics. Got `Err(Semantic)`: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => {
            panic!("unexpected compile error for {R2_PR_A_RUNTIME_VALUE_FIXTURE_PATH}: {other:?}")
        }
    };

    let results = TestRunner::new(&dag).run_suite("r2_evaluator_runtime_value_model_suite");
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].claim_name,
        "evaluator_runtime_value_model_landed"
    );
    assert_eq!(results[0].result, ClaimResult::Pass);
}

/// Tier-3 `PerfWithinBaseline` substrate smoke (`p99_delta_ns` + `AtMostBudget`).
const R3_PERF_WITHIN_BASELINE_SMOKE_FIXTURE: &str =
    include_str!("../fixtures/r3_perf_within_baseline_smoke.dag");
const R3_PERF_WITHIN_BASELINE_SMOKE_FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/r3_perf_within_baseline_smoke.dag";

#[test]
fn r3_perf_within_baseline_smoke_suite_passes() {
    let dag = match compile_to_dag(
        R3_PERF_WITHIN_BASELINE_SMOKE_FIXTURE,
        R3_PERF_WITHIN_BASELINE_SMOKE_FIXTURE_PATH,
    ) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "{R3_PERF_WITHIN_BASELINE_SMOKE_FIXTURE_PATH}: semantic compile error: {:?}",
            dag.diagnostics()
        ),
        Err(e) => panic!("{R3_PERF_WITHIN_BASELINE_SMOKE_FIXTURE_PATH}: unexpected error: {e:?}"),
    };
    assert!(
        dag.diagnostics().is_empty(),
        "{R3_PERF_WITHIN_BASELINE_SMOKE_FIXTURE_PATH}: expected no diagnostics, got {:?}",
        dag.diagnostics()
    );
    let results = TestRunner::new(&dag).run_suite("perf_within_smoke_suite");
    assert_eq!(results.len(), 1, "expected one claim");
    assert_eq!(
        results[0].result,
        ClaimResult::Pass,
        "claim {:?}",
        results[0].claim_name
    );
}

/// TC2 (Evaluator Manager): evaluation-order independence theorem shape.
/// **Author-now-fire-later** `BinaryDimensionReportEquals` consumer (unified predicate
/// PR #1318) with strategy-order role declarations; runner report equality is NYI until
/// `DimensionReport<C>` production lands. Strict-fire still waits on a second executable
/// strategy (PR #1316 §4 P4).
const TC2_EVALUATION_ORDER_FIXTURE: &str =
    include_str!("../fixtures/tc2_evaluation_order_independence_deferred.dag");
const TC2_EVALUATION_ORDER_FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag";
// Test-harness containment: this fixture compiles enough bootstrap/runtime surface to
// overflow the default harness stack in CI. Dissolution trigger: remove or centralize
// this wrapper once the TC2 fixture runs on the default stack.
const TC2_FIXTURE_TEST_STACK_BYTES: usize = 32 * 1024 * 1024;

#[test]
fn tc2_evaluation_order_independence_suite_passes() {
    std::thread::Builder::new()
        .stack_size(TC2_FIXTURE_TEST_STACK_BYTES)
        .spawn(tc2_evaluation_order_independence_suite_passes_impl)
        .expect("spawn larger-stack TC2 fixture test thread")
        .join()
        .expect("larger-stack TC2 fixture test thread panicked");
}

fn tc2_evaluation_order_independence_suite_passes_impl() {
    let dag = match compile_to_dag(TC2_EVALUATION_ORDER_FIXTURE, TC2_EVALUATION_ORDER_FIXTURE_PATH)
    {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{TC2_EVALUATION_ORDER_FIXTURE_PATH}: expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{TC2_EVALUATION_ORDER_FIXTURE_PATH} should lower without module diagnostics. Got `Err(Semantic)`: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => {
            panic!("unexpected compile error for {TC2_EVALUATION_ORDER_FIXTURE_PATH}: {other:?}")
        }
    };

    let results = TestRunner::new(&dag).run_suite("tc2_evaluation_order_independence_suite");
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].claim_name,
        "evaluation_order_independent_lens_results"
    );
    // Predicate is `BinaryDimensionReportEquals` only; at head the runner returns NYI after
    // shape validation. Do not assert on `reason` substrings (TESTING.md: avoid pinning message text).
    assert!(
        matches!(&results[0].result, ClaimResult::NotYetImplemented(_)),
        "expected TC2 claim to stop at NYI (shape-valid `BinaryDimensionReportEquals`), got {:?}",
        results[0].result
    );
}

const BRIDGE_LEDGER_ZERO_SOURCE: &str =
    include_str!("../fixtures/r3_bridge_retirement_ledger_zero.dag");
const BRIDGE_LEDGER_ZERO_PATH: &str =
    "src/v3/compiler/tests/fixtures/r3_bridge_retirement_ledger_zero.dag";

// Performance caches only: these amortize fixture/bootstrap compilation across
// tests and are not part of the verification model or claim authority.
static BRIDGE_LEDGER_ZERO_DAG: OnceLock<Dag> = OnceLock::new();

fn bridge_ledger_zero_dag() -> &'static Dag {
    BRIDGE_LEDGER_ZERO_DAG.get_or_init(|| {
        compile_to_dag(BRIDGE_LEDGER_ZERO_SOURCE, BRIDGE_LEDGER_ZERO_PATH)
            .expect("bridge ledger zero fixture compiles")
    })
}

const RUST_DAG_ISOMORPHISM_SOURCE: &str =
    include_str!("../fixtures/rust_dag_isomorphism_consumer.dag");
const RUST_DAG_ISOMORPHISM_PATH: &str =
    "src/v3/compiler/tests/fixtures/rust_dag_isomorphism_consumer.dag";
static RUST_DAG_ISOMORPHISM_DAG: OnceLock<Dag> = OnceLock::new();

fn rust_dag_isomorphism_dag() -> &'static Dag {
    RUST_DAG_ISOMORPHISM_DAG.get_or_init(|| {
        compile_to_dag(RUST_DAG_ISOMORPHISM_SOURCE, RUST_DAG_ISOMORPHISM_PATH)
            .expect("RustDagIsomorphism consumer fixture compiles")
    })
}

static BRIDGE_LEDGER_OPEN_ROW_NAMES: OnceLock<Vec<String>> = OnceLock::new();
/// BridgeLedgerZero open-row bound is monotone non-increasing; PRs that decrease this
/// count retire bridges, and PRs that need to increase it must update
/// `r3-v-bridge-ratchet-test-design.md` §Per-Bridge Gate Audit and obtain
/// Verification-Mgr acknowledgment.
const EXPECTED_OPEN_BOUND: usize = 4;

fn bridge_ledger_open_row_names() -> &'static [String] {
    BRIDGE_LEDGER_OPEN_ROW_NAMES.get_or_init(|| {
        let dag = generated_full_bootstrap_dag();
        let retired_constructor = {
            let bridge_status = dag
                .declaration_by_name("BridgeStatus")
                .expect("BridgeStatus missing from full bootstrap");
            let TypeConnective::Disj { variants } = &bridge_status.connective else {
                panic!("BridgeStatus is not a Disj");
            };
            variants
                .iter()
                .find(|variant| variant.label == "Retired")
                .expect("Retired variant missing")
                .ty
        };
        let bridge_ledger = dag
            .declaration_by_name("bridge_ledger")
            .expect("bridge_ledger missing from full bootstrap");
        let Some(ValueBody::List(rows)) = &bridge_ledger.value_body else {
            panic!("bridge_ledger must lower as a List value body");
        };

        rows.iter()
            .filter_map(|row| {
                let FieldValue::Record(fields) = row else {
                    panic!("bridge_ledger row is not a record: {row:?}");
                };
                let constructor = match record_field(fields, "status") {
                    FieldValue::Variant { constructor, .. } => *constructor,
                    other => panic!("bridge_ledger status is not a variant: {other:?}"),
                };
                if constructor == retired_constructor {
                    None
                } else {
                    Some(string_literal(record_field(fields, "name")).to_string())
                }
            })
            .collect()
    })
}

fn record_field<'a>(fields: &'a [(String, FieldValue)], label: &str) -> &'a FieldValue {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("record missing `{label}` field"))
}

fn string_literal(value: &FieldValue) -> &str {
    match value {
        FieldValue::Literal(LiteralBits::String(s)) => s.as_str(),
        other => panic!("expected String literal, got {other:?}"),
    }
}

#[test]
fn r3_bridge_retirement_ledger_zero_open_row_count_ratchet() {
    let results = TestRunner::new(bridge_ledger_zero_dag())
        .run_suite("r3_bridge_retirement_ledger_zero_suite");
    assert_eq!(results.len(), 1);
    let reason = match &results[0].result {
        ClaimResult::Fail(reason) => reason,
        other => panic!("expected bridge ledger zero to be red at HEAD; got {other:?}"),
    };

    let open_rows = bridge_ledger_open_row_names();
    assert!(
        open_rows.len() <= EXPECTED_OPEN_BOUND,
        "BridgeLedgerZero decreasing-open-count ratchet: current open-row count {} \
         exceeds recorded bound {}; open rows: [{}]",
        open_rows.len(),
        EXPECTED_OPEN_BOUND,
        open_rows.join(", ")
    );
    assert!(
        !open_rows.is_empty(),
        "when the canonical bridge ledger reaches zero open rows, re-arm this \
         fixture expectation as a Pass ratchet in the same PR"
    );
    for row in open_rows {
        assert!(
            reason.contains(row),
            "BridgeLedgerZero diagnostic must name open row `{row}`; got: {reason}"
        );
    }
}

#[test]
fn rust_dag_isomorphism_executable_passes_dag_shape_report_gate() {
    // P5 per-PR receipt for this expanded hand-Rust test harness: explicit deferral to
    // ROADMAP.md "Course correction #1" / table row 1 ("One BinaryDimensionReportEquals
    // path executes"). This test must continue to stop at the shape-valid NYI boundary
    // until the generated DimensionReport/DagShapeReport path replaces the Rust harness.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            let results = TestRunner::new(rust_dag_isomorphism_dag())
                .run_suite("rust_dag_isomorphism_consumer_suite");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].claim_name, "rust_dag_isomorphism_executable");
            assert!(
                matches!(
                    &results[0].result,
                    ClaimResult::NotYetImplemented(reason)
                        if reason.contains("BinaryDimensionReportEquals")
                            && reason.contains("structural shape is valid")
                            && reason.contains("RustEnumExtractionDagShapeReport")
                            && reason.contains("DagReflectionDagShapeReport")
                ),
                "expected RustDagIsomorphism executable to stop at the \
                 BinaryDimensionReportEquals shape-valid boundary; got {:?}",
                results[0].result
            );
        })
        .expect("spawn rust_dag_isomorphism stack")
        .join()
        .expect("rust_dag_isomorphism gate thread");
}

/// R3 gate #40 (`symbolic_cost_expr_equals_executable`): `SymbolicCostExprEquals` applies the
/// symbolic-cost lens to `TestClaim.source` and compares against the declared expected `SymbolicCost`.
const SYMBOLIC_COST_EXPR_EQUALS_SMOKE_FIXTURE: &str = r#"
module std.symbolic_cost_expr_equals_smoke

import std.verification {
  SymbolicCostExprEquals,
  TestClaim,
  TestSuite
}
import std.algebra { SymbolicCost }

data expected_lit_cost: SymbolicCost = ConstantCost(0)

data symbolic_cost_expr_equals_claim: TestClaim = {
  name: "literal_symbolic_cost_matches_expected",
  source: "let lit: Int = 7",
  file_name: "lit.v3",
  predicate: SymbolicCostExprEquals(expected_lit_cost),
  requires: []
}

data symbolic_cost_expr_equals_suite: TestSuite = {
  name: "symbolic_cost_expr_equals_smoke_suite",
  claims: [symbolic_cost_expr_equals_claim]
}
"#;

#[test]
fn symbolic_cost_expr_equals_smoke_suite_passes() {
    let dag = match compile_to_dag(
        SYMBOLIC_COST_EXPR_EQUALS_SMOKE_FIXTURE,
        "symbolic_cost_expr_equals_smoke.dag",
    ) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "fixture should compile cleanly: {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "fixture semantic failure: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("fixture compile error: {other:?}"),
    };
    let results = TestRunner::new(&dag).run_suite("symbolic_cost_expr_equals_suite");
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].claim_name,
        "literal_symbolic_cost_matches_expected"
    );
    assert_eq!(results[0].result, ClaimResult::Pass);
}
