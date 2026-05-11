//! **Layer:** boundary
//!
//! R3 gate #71 — `v3_self_host_demonstration` (T-V2-Retirement).
//!
//! [`compile_to_dag`] + [`TestRunner`] evaluates a [`TestClaim`] whose predicate is
//! [`ExecuteCommand`] over `self_host_fixed_point --r3-gate-71-demonstration` — PB-Runtime
//! trampoline plus a **strict** DB-8 slice: the binary exits non-zero unless
//! `dsl/gunbc/compiler.dag` parses under v3 and `receipt.json` records `fixed_point_diff` → `ok`.
//! Default `self_host_fixed_point` (no flag) stays staged per DB-8.
//!
//! The `.dag` template carries `__SELF_HOST_FIXED_POINT_BIN__`; substitution uses
//! `env!("CARGO_BIN_EXE_self_host_fixed_point")` at integration-test compile time (R1
//! Closure `#973` discipline, parallel to `t_pb_b_1_dag_runner_test` / `r1c_e_emit_gates.template.dag`).
//!
//! **Staging:** while `compiler.dag` still fails v3 parse, this end-to-end test stays `#[ignore]`.
//! **Toolchain:** full Rust + `rustc` when the parse + emit slice runs.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const TEMPLATE: &str = include_str!("../dag/r3_v3_self_host_demonstration.template.dag");
const TEMPLATE_PATH: &str = "src/v3/compiler/tests/dag/r3_v3_self_host_demonstration.template.dag";
const BIN_PLACEHOLDER: &str = "__SELF_HOST_FIXED_POINT_BIN__";

fn substituted_dag_source() -> String {
    let bin_path = env!("CARGO_BIN_EXE_self_host_fixed_point");
    assert!(
        TEMPLATE.contains(BIN_PLACEHOLDER),
        "template must contain `{BIN_PLACEHOLDER}` placeholder for bin substitution \
         (see `r1c_e_emit_gates.template.dag` discipline): {TEMPLATE_PATH}"
    );
    TEMPLATE.replace(BIN_PLACEHOLDER, bin_path)
}

/// Structural compile-smoke: template lowers with substituted bin path (always on in CI).
#[test]
fn r3_v3_self_host_demonstration_dag_lowers_with_substituted_bin_path() {
    let source = substituted_dag_source();
    match compile_to_dag(&source, TEMPLATE_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{TEMPLATE_PATH} (substituted): expected empty module diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "{TEMPLATE_PATH} (substituted) should lower without module diagnostics. \
                 Got `Err(Semantic)`: {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
        }
        Err(other) => panic!("unexpected compile error for {TEMPLATE_PATH}: {other:?}"),
    }
}

/// R3 §1.8 gate #71 — strict demonstration (PB-Runtime + full self-host slice). Ignored until
/// `dsl/gunbc/compiler.dag` parses under v3 (`--r3-gate-71-demonstration` contract).
#[test]
#[ignore = "Unignore when v3 parses dsl/gunbc/compiler.dag and DB-8 slice reaches fixed_point_diff ok (T-FixedPoint / Lane 3 promotion per design-fixed-point-ratchet.md)."]
fn r3_v3_self_host_demonstration_suite_passes_through_runner() {
    let source = substituted_dag_source();
    let dag = match compile_to_dag(&source, TEMPLATE_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{TEMPLATE_PATH} (substituted): expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "{TEMPLATE_PATH} (substituted) should lower without module diagnostics. \
                 Got `Err(Semantic)`: {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
        }
        Err(other) => panic!("unexpected compile error for {TEMPLATE_PATH}: {other:?}"),
    };

    let results = TestRunner::new(&dag).run_suite("suite_v3_self_host_demonstration");
    assert!(
        !results.is_empty(),
        "suite `suite_v3_self_host_demonstration` should contain at least one claim"
    );
    let failures: Vec<_> = results
        .iter()
        .filter(|r| r.result != ClaimResult::Pass)
        .collect();
    assert!(
        failures.is_empty(),
        "suite_v3_self_host_demonstration: {} claim(s) did not Pass:\n{:#?}",
        failures.len(),
        failures
    );
}
