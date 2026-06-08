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
//!
//! **R3 Cluster M #84 — R1C-D/E tests-as-data pilot:** R1C-D (`t_r1c_d_pb_census_gates.dag`)
//! and R1C-E (`r1c_e_emit_gates.template.dag` + omni template) integration receipts
//! live here with gate #74 (`t_r3_tests_as_data_demonstration.dag`), replacing dedicated
//! `r1c_*_test.rs` shims. **Accounting:** SG-0 progress for #2715 is the **−3** census paths
//! plus `.dag`-native predicates — not gate #84 / facet-3 'zero hand-Rust tests' closure; see
//! `sg0_census_test.rs` on `t_pb_b_1_dag_runner_test.rs` (remaining obligation cites ROADMAP
//! T-PB-B until the test census is empty).
//!
//! R3 gate #87 `R3_GATE_87_CEMENTING_REGEN_SUITES` wiring: **INVARIANTS P5(b)** — merge-visible
//! integration delta; see module doc on `r3_gate_87_lens_cementing_regen_receipts_test` (§P5(b)
//! checkable receipt = **PR #2639 description**, not inferred deletes). Table lives in
//! `v3_compiler::r3_gate_87_cementing_regen_runner_suites` (shared with `cementing_dispatch`).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::r3_gate_87_cementing_regen_runner_suites::R3_GATE_87_CEMENTING_REGEN_SUITES;
use v3_compiler::test_runner::{ClaimResult, TestClaimValue, TestRunner};
use v3_compiler::CompileError;

fn lower(source: &str, file: &str) -> Dag {
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

fn claim_value_by_decl_name(dag: &Dag, declaration_name: &str) -> TestClaimValue {
    let decl = dag
        .declaration_by_name(declaration_name)
        .unwrap_or_else(|| panic!("expected TestClaim declaration `{declaration_name}`"));
    TestClaimValue::from_declaration(decl)
        .unwrap_or_else(|reason| panic!("`{declaration_name}` should lower as TestClaim: {reason}"))
}

/// R1C-D receipt: six PB census claims must dispatch to wired evaluators (no `NotYetImplemented`)
/// and evaluate to structural `Pass` or `Fail` — same acceptance as the former
/// `r1c_d_pb_census_gates_test` shim (`docs/briefs/r1-closure-manager.md` §R1C-D).
fn run_suite_r1c_d_pb_census_receipt(dag: &Dag, suite_name: &str) {
    const EXPECTED_CLAIM_NAMES: &[&str] = &[
        "pb_hand_rust_at_shim_floor",
        "lens_producer_files_remaining",
        "pb_self_compile_fixed_point",
        "pb_compiler_std_ratchet_zero",
        "pb_test_file_generated_from_dag",
        "pb_rust_tests_outside_residual_zero",
    ];
    let results = TestRunner::new(dag).run_suite(suite_name);
    let actual_names: Vec<&str> = results.iter().map(|r| r.claim_name.as_str()).collect();
    assert_eq!(
        actual_names, EXPECTED_CLAIM_NAMES,
        "suite `{suite_name}`: claim order must match the declared deliverable list"
    );
    let unimplemented: Vec<_> = results
        .iter()
        .filter(|r| matches!(r.result, ClaimResult::NotYetImplemented(_)))
        .collect();
    assert!(
        unimplemented.is_empty(),
        "PB census predicates must dispatch to wired evaluators, not `NotYetImplemented`. Offenders:\n{unimplemented:#?}"
    );
    for result in &results {
        match &result.result {
            ClaimResult::Pass | ClaimResult::Fail(_) => {}
            other => panic!(
                "PB census claim `{}` must evaluate to Pass or Fail, got {:?}",
                result.claim_name, other
            ),
        }
    }
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

#[test]
fn r3_fieldproject_dual_authority_dissolution_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_r3_fieldproject_dual_authority_dissolution.dag"),
        "src/v3/compiler/tests/dag/t_r3_fieldproject_dual_authority_dissolution.dag",
    );
    run_suite_all_pass(&dag, "suite_fieldproject_dual_authority_dissolution");
}

#[test]
fn r3_missing_emission_path_typed_axes_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_r3_missing_emission_path_typed_axes.dag"),
        "src/v3/compiler/tests/dag/t_r3_missing_emission_path_typed_axes.dag",
    );
    run_suite_all_pass(&dag, "suite_missing_emission_path_typed_axes");
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

/// R3 gate #74 — one Rust integration test ported to `.dag` `TestClaim` data and
/// executed end-to-end through `TestRunner`.
///
/// Port target: retired `t_pb_b_brief_d_pipeline_smoke_fixture_lowers_cleanly`
/// (`t_pb_b_brief_d_fixture_smoke_test.rs`). The original Rust test asserted the
/// pipeline smoke fixture lowers cleanly; this carrier expresses the same
/// surface as a `Compiles` claim over the embedded subject program.
#[test]
fn r3_tests_as_data_demonstration_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_r3_tests_as_data_demonstration.dag"),
        "src/v3/compiler/tests/dag/t_r3_tests_as_data_demonstration.dag",
    );
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "suite_tests_as_data_demonstration",
        &["tests-as-data port of pipeline smoke fixture compiles"],
    );

    let baseline = lower(
        include_str!("../dag/t_pb_b_1_pipeline_smoke.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_pipeline_smoke.dag",
    );
    let baseline_claim = claim_value_by_decl_name(&baseline, "claim_pipe_unary_compiles");
    let demonstration_claim =
        claim_value_by_decl_name(&dag, "claim_tests_as_data_pipeline_smoke_compiles");
    assert_eq!(
        demonstration_claim.source, baseline_claim.source,
        "gate #74 demonstration claim must stay byte-aligned with the pipeline-smoke subject"
    );
    assert_eq!(
        demonstration_claim.file_name, baseline_claim.file_name,
        "gate #74 demonstration claim must keep the same subject file authority"
    );
}

const R1C_D_PB_CENSUS_GATES_PATH: &str = "src/v3/compiler/tests/dag/t_r1c_d_pb_census_gates.dag";
const R1C_D_PB_CENSUS_SUITE: &str = "r1_pb_census_gates_suite";

/// R1C-D integration receipt: the six PB census gates are `TestClaim` + predicate **data** in
/// `t_r1c_d_pb_census_gates.dag` (path: `R1C_D_PB_CENSUS_GATES_PATH`), not in this Rust body. This
/// `#[test]` only lowers the module and runs the suite through `TestRunner` (runner-only, same
/// class as gate #74). **P5(b):** merge-visible SG-0 receipt is **−3** deleted census paths; **not**
/// gate #84 / facet-3 closure — see `sg0_census_test.rs` on this file's `EXPECTED_HAND_AUTHORED_TEST`
/// line and the module doc above.
#[test]
fn r1c_d_pb_census_gates_suite_evaluates_through_runner() {
    let dag = lower(
        include_str!("../dag/t_r1c_d_pb_census_gates.dag"),
        R1C_D_PB_CENSUS_GATES_PATH,
    );
    run_suite_r1c_d_pb_census_receipt(&dag, R1C_D_PB_CENSUS_SUITE);
}

const R1C_E_EMIT_GATES_TEMPLATE: &str = include_str!("../dag/r1c_e_emit_gates.template.dag");
const R1C_E_EMIT_GATES_TEMPLATE_PATH: &str =
    "src/v3/compiler/tests/dag/r1c_e_emit_gates.template.dag";
const R1C_E_EMIT_GATES_BIN_PATH: &str = env!("CARGO_BIN_EXE_r1c_e_emit_gates");
const R1C_E_BIN_PLACEHOLDER: &str = "__R1C_E_BIN__";

fn substituted_r1c_e_emit_gates_source() -> String {
    assert!(
        R1C_E_EMIT_GATES_TEMPLATE.contains(R1C_E_BIN_PLACEHOLDER),
        "template must contain `{R1C_E_BIN_PLACEHOLDER}` placeholder for bin substitution \
         (see manager guidance, #973): {R1C_E_EMIT_GATES_TEMPLATE_PATH}"
    );
    R1C_E_EMIT_GATES_TEMPLATE.replace(R1C_E_BIN_PLACEHOLDER, R1C_E_EMIT_GATES_BIN_PATH)
}

/// R1C-E: emit-gate claims live in `r1c_e_emit_gates.template.dag` (host path substituted in
/// `substituted_r1c_e_emit_gates_source`). Runner-only wiring; same P5(b) / #84 accounting as
/// `r1c_d_pb_census_gates_suite_evaluates_through_runner`.
#[test]
fn r1c_e_emit_gates_suite_passes_through_runner() {
    let source = substituted_r1c_e_emit_gates_source();
    let dag = lower(&source, R1C_E_EMIT_GATES_TEMPLATE_PATH);

    let results = TestRunner::new(&dag).run_suite("r1c_e_emit_gates_suite");
    assert!(
        !results.is_empty(),
        "suite `r1c_e_emit_gates_suite` should contain at least one claim"
    );
    let failures: Vec<_> = results
        .iter()
        .filter(|r| r.result != ClaimResult::Pass)
        .collect();
    assert!(
        failures.is_empty(),
        "r1c_e_emit_gates_suite: {} claim(s) did not Pass:\n{:#?}",
        failures.len(),
        failures
    );
}

const R1C_E_OMNI_TEMPLATE: &str = include_str!("../dag/r1c_e_emit_gates_omni.template.dag");
const R1C_E_OMNI_TEMPLATE_PATH: &str =
    "src/v3/compiler/tests/dag/r1c_e_emit_gates_omni.template.dag";

fn substituted_r1c_e_emit_gates_omni_source() -> String {
    assert!(
        R1C_E_OMNI_TEMPLATE.contains(R1C_E_BIN_PLACEHOLDER),
        "omni template must contain `{R1C_E_BIN_PLACEHOLDER}`: {R1C_E_OMNI_TEMPLATE_PATH}"
    );
    R1C_E_OMNI_TEMPLATE.replace(R1C_E_BIN_PLACEHOLDER, R1C_E_EMIT_GATES_BIN_PATH)
}

/// Multi-target omni emit claim — requires **go** and **python3** on `PATH` (ignored in default CI).
#[test]
#[ignore]
fn r1c_e_emit_gates_omni_suite_passes() {
    let source = substituted_r1c_e_emit_gates_omni_source();
    let dag = lower(&source, R1C_E_OMNI_TEMPLATE_PATH);

    let results = TestRunner::new(&dag).run_suite("r1c_e_emit_gates_omni_suite");
    assert!(
        !results.is_empty(),
        "suite `r1c_e_emit_gates_omni_suite` should have claims"
    );
    let failures: Vec<_> = results
        .iter()
        .filter(|r| r.result != ClaimResult::Pass)
        .collect();
    assert!(
        failures.is_empty(),
        "r1c_e_emit_gates_omni_suite: {} claim(s) did not Pass:\n{:#?}",
        failures.len(),
        failures
    );
}

const BOUNDARY_EMIT_GATES_TEMPLATE: &str = include_str!("../dag/boundary_emit_gates.template.dag");
const BOUNDARY_EMIT_GATES_TEMPLATE_PATH: &str =
    "src/v3/compiler/tests/dag/boundary_emit_gates.template.dag";
const BOUNDARY_EMIT_GATES_BIN_PATH: &str = env!("CARGO_BIN_EXE_boundary_emit_gates");
const BOUNDARY_EMIT_BIN_PLACEHOLDER: &str = "__BOUNDARY_EMIT_BIN__";

fn substituted_boundary_emit_gates_source() -> String {
    assert!(
        BOUNDARY_EMIT_GATES_TEMPLATE.contains(BOUNDARY_EMIT_BIN_PLACEHOLDER),
        "template must contain `{BOUNDARY_EMIT_BIN_PLACEHOLDER}`: {BOUNDARY_EMIT_GATES_TEMPLATE_PATH}"
    );
    BOUNDARY_EMIT_GATES_TEMPLATE
        .replace(BOUNDARY_EMIT_BIN_PLACEHOLDER, BOUNDARY_EMIT_GATES_BIN_PATH)
}

/// F.14 / T-PB-B: class-5 boundary emit gates as `.dag` `TestClaim` data (m2 + python division).
#[test]
fn boundary_emit_gates_suite_passes_through_runner() {
    let source = substituted_boundary_emit_gates_source();
    let dag = lower(&source, BOUNDARY_EMIT_GATES_TEMPLATE_PATH);
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "boundary_emit_gates_suite",
        &[
            "multi_field_struct_variant_match_emits_aliased_field_destructure",
            "multi_field_struct_variant_arm_body_uses_aliased_reference",
            "multi_field_struct_variant_emitted_rust_is_valid_syntax",
            "emit_python_checked_division_roundtrips_ok_and_errors",
        ],
    );
}

/// R3 gate #65 — a Tier3 mirror-consumer `.dag` program executes through
/// `TestRunner` using std termination authority instead of the hand-Rust mirror
/// bench path.
#[test]
fn r3_tier3_dissolution_demonstration_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_r3_tier3_dissolution_demonstration.dag"),
        "src/v3/compiler/tests/dag/t_r3_tier3_dissolution_demonstration.dag",
    );
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "suite_tier3_dissolution_demonstration",
        &["tier3_dissolution_demonstration_executes"],
    );
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
fn t_impossiblebugs_release_demo_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_impossiblebugs_release_demos.dag"),
        "src/v3/compiler/tests/dag/t_impossiblebugs_release_demos.dag",
    );
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "suite_impossible_bug_release_demos",
        &[
            "impossible_bug_release_demo.complexity_over_bound_lens",
            "impossible_bug_release_demo.idempotency_loop_lens",
            "impossible_bug_release_demo.transport_type_drift",
        ],
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

// R3 gate #74 (`tests_as_data_demonstration`) — executable `.dag` `TestClaim` + runner receipt
// lives in `r3_tests_as_data_demonstration_suite_passes_through_runner` above (and
// `tests/dag/t_r3_tests_as_data_demonstration.dag` on `main`). Gate-#87 regen harnesses below are a
// separate ratchet; do not conflate the two in PR titles or census expectations.

#[test]
fn r3_gate_87_cementing_regen_lens_suites_pass_through_runner() {
    // One `compile_to_dag` per `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag`
    // harness — the on-disk file is the single authority for that row (INVARIANTS P2); no
    // duplicated copy in a bundle module.
    for (source, file, suite, claim_names) in R3_GATE_87_CEMENTING_REGEN_SUITES {
        let dag = lower(source, file);
        run_suite_all_pass_with_expected_claim_names(&dag, suite, claim_names);
    }
}

#[test]
fn cementing_dispatch_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/cementing_dispatch.dag"),
        "src/v3/compiler/tests/dag/cementing_dispatch.dag",
    );
    run_suite_all_pass_with_expected_claim_names(
        &dag,
        "cementing_dispatch_suite",
        &["cementing_dispatch_projection_matches_register_and_regen"],
    );
}
