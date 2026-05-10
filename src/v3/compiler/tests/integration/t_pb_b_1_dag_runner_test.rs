//! **Layer:** integration
//!
//! T-PB-B-1 runner wiring — proves `TestRunner` evaluates the landed
//! `src/v3/compiler/tests/dag/*.dag` `TestSuite` modules through the same
//! `compile_to_dag` entrypoint as the rest of the integration harness. This is
//! line-item (1) of the pre–Rust-deletion checklist in
//! `docs/briefs/r1-testgen-manager.md` (Hand-off → Self-hosting): runner path
//! accepts the landed layout and `requires: []` lowers to a shape the runner
//! consumes. `lower` requires `compile_to_dag` → `Ok` (empty module diagnostics
//! per `lib.rs`); `Err(Semantic(_))` panics with the same compile-smoke intent as
//! the retired `t_pb_b_1_tests_dag_smoke_test` (runner evaluation of embedded
//! `TestClaim.source` is orthogonal). Still
//! **not** a `pb_*` gate and still not a Rust-deletion signal.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

fn lower(source: &'static str, file: &'static str) -> Dag {
    // `compile_to_dag` returns `Ok` iff the module diagnostic table is empty
    // (`lib.rs` — any semantic issue is `Err(Semantic(dag))` with non-empty
    // diagnostics). The retired `t_pb_b_1_tests_dag_smoke_test` required the
    // same: declaring `tests/dag` harness compiles with no module diagnostics.
    match compile_to_dag(source, file) {
        Ok(dag) => {
            // Explicit compile-smoke receipt: same as the retired
            // `t_pb_b_1_tests_dag_smoke_test` (in addition to `Ok` ⟺ empty in `lib.rs`).
            // Belt-and-suspenders against C-8 / `compile_to_dag` contract drift; unreachable
            // if `Ok` truly implies an empty table — kept so reviewers and grep see the
            // harness receipt without relying only on the `Result` shape.
            assert!(
                dag.diagnostics().is_empty(),
                "{file} (declaring `tests/dag` harness): expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "{file} (declaring `tests/dag` harness) should lower without module diagnostics — \
                 same compile-smoke receipt as the retired `t_pb_b_1_tests_dag_smoke_test`. \
                 Got `Err(Semantic)`: {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
        }
        Err(other) => panic!("unexpected compile error for {file}: {other:?}"),
    }
}

fn run_suite_all_pass(dag: &Dag, suite_name: &str) {
    let results = TestRunner::new(dag).run_suite(suite_name);
    assert!(
        !results.is_empty(),
        "suite `{suite_name}` should contain at least one claim"
    );
    assert!(
        results
            .iter()
            .all(|result| result.result == ClaimResult::Pass),
        "suite `{suite_name}` should pass every claim, got {results:?}"
    );
}

/// Deletion-guard: assert suite cardinality, **claim `name` order** (structural
/// `TestClaim` identity, not `data` decl id), and all `Pass` — the receipt carried
/// by the retired per-gate `#[test]` shims.
fn run_suite_all_pass_with_expected_claim_names(
    dag: &Dag,
    suite_name: &str,
    expected_claim_names: &[&str],
) {
    let results = TestRunner::new(dag).run_suite(suite_name);
    let actual: Vec<&str> = results.iter().map(|e| e.claim_name.as_str()).collect();
    let expected: Vec<&str> = expected_claim_names.to_vec();
    assert_eq!(
        actual, expected,
        "suite `{suite_name}`: expected claim `name` list (structural order) to match"
    );
    assert!(
        !results.is_empty() && results.iter().all(|r| r.result == ClaimResult::Pass),
        "suite `{suite_name}`: expected every claim to pass, got {results:?}"
    );
}

#[test]
fn t_pb_b_1_pipeline_smoke_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_pb_b_1_pipeline_smoke.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_pipeline_smoke.dag",
    );
    run_suite_all_pass(&dag, "suite_pipeline_pipe_unary");
}

#[test]
fn t_pb_b_1_contract_diagnostic_smoke_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_pb_b_1_contract_diagnostic_smoke.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_contract_diagnostic_smoke.dag",
    );
    run_suite_all_pass(&dag, "suite_contract_diagnostic_negatives");
}

#[test]
fn t_pb_b_1_contract_port_cost_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_pb_b_1_contract_port_cost.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_contract_port_cost.dag",
    );
    run_suite_all_pass(&dag, "suite_contract_port_and_cost");
}

/// `ExecuteCommand` through the same `tests/dag` path as T-PB-B-1 (PB-Runtime extension).
/// Boundary migration: `m1_4_emit_python_test::python_stdout` pattern → declarative
/// `ExecuteCommand` (this suite uses `sh`/`echo` / `true` so CI need not install CPython).
/// Repository CI (`.github/workflows/ci.yml` `v3` job) is Linux-only; on Windows, `sh`/`echo` may
/// be absent or diverge — this test is not run on Windows in CI today. If the matrix **adds** a
/// Windows (or other non-POSIX) target for `v3` without gating, expect this test and the
/// `sh`/`echo` claims in the `.dag` to be the first break (api-review e99b53e7).
#[test]
fn t_pb_b_1_execute_command_boundary_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_pb_b_1_execute_command_boundary.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_execute_command_boundary.dag",
    );
    run_suite_all_pass(&dag, "suite_execute_command_boundary");
}

#[test]
fn t_impossiblebugs_nested_optional_flatten_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_impossiblebugs_nested_optional_flatten.dag"),
        "src/v3/compiler/tests/dag/t_impossiblebugs_nested_optional_flatten.dag",
    );
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "suite_nested_optional_flatten_class_close",
        &["nested_optional_flatten_compile_error"],
    );
}

#[test]
fn t_impossiblebugs_force_unwrap_regression_verify_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_impossiblebugs_force_unwrap_regression_verify.dag"),
        "src/v3/compiler/tests/dag/t_impossiblebugs_force_unwrap_regression_verify.dag",
    );
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "suite_force_unwrap_regression_verify",
        &["force_unwrap_absent_resolve_error"],
    );
}

#[test]
fn r3_callgraph_forward_only_authority_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/r3_callgraph_forward_only_authority.dag"),
        "src/v3/compiler/tests/dag/r3_callgraph_forward_only_authority.dag",
    );
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "suite_r3_callgraph_forward_only_authority",
        &["callgraph_derived_adjacency_views_coherent"],
    );
}

/// R1 gate suites from `tests/fixtures/r1_gates.dag` — same `TestClaim` authority and
/// exact `claim_name` receipt as the retired
/// `r1_manual_claim_gate_test` / `testgen_structural_coverage_gate_test` shims.
#[test]
fn r1_gates_manual_claim_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../fixtures/r1_gates.dag"),
        "src/v3/compiler/tests/fixtures/r1_gates.dag",
    );
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "manual_claim_suite",
        &["testgen_manual_claim_is_first_class"],
    );
}

#[test]
fn r1_gates_lens_composition_associative_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../fixtures/r1_gates.dag"),
        "src/v3/compiler/tests/fixtures/r1_gates.dag",
    );
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "lens_composition_associative_suite",
        &["lens_composition_associative"],
    );
}

#[test]
fn r1_gates_testgen_structural_coverage_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../fixtures/r1_gates.dag"),
        "src/v3/compiler/tests/fixtures/r1_gates.dag",
    );
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "testgen_structural_coverage_suite",
        &["testgen_structural_coverage"],
    );
}
