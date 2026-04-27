//! **Layer:** integration
//!
//! R1C-E — `emit_omni_demo_fixtures_green` `.dag` `TestClaim` (subcommand
//! `omni-demo` on the `r1c_e_emit_gates` bin). This suite requires **go** and
//! **python3**; normal CI does not install them, so the receipt is `#[ignore]`d
//! (same as the host `m1_5_emit_omni_demo_test::emit_omni_demo_fixtures_green`).
//! The main R1C-E template (`r1c_e_emit_gates.template.dag`) stays unignored
//! and covers `generic-bounds` + `rust-fixtures` only.
//!
//! Run locally (after `r1c_e_emit_gates` is built, e.g. `cargo test -p v3-compiler --test integration`):
//! `cargo test -p v3-compiler --test integration r1c_e_emit_gates_omni_suite_passes -- --ignored --nocapture`

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const TEMPLATE: &str = include_str!("../dag/r1c_e_emit_gates_omni.template.dag");
const TEMPLATE_PATH: &str = "src/v3/compiler/tests/dag/r1c_e_emit_gates_omni.template.dag";
const BIN_PATH: &str = env!("CARGO_BIN_EXE_r1c_e_emit_gates");
const BIN_PLACEHOLDER: &str = "__R1C_E_BIN__";

fn substituted_dag_source() -> String {
    assert!(
        TEMPLATE.contains(BIN_PLACEHOLDER),
        "template must contain `{BIN_PLACEHOLDER}` placeholder for bin substitution: {TEMPLATE_PATH}"
    );
    TEMPLATE.replace(BIN_PLACEHOLDER, BIN_PATH)
}

#[test]
#[ignore = "requires go and python3 on PATH (same as emit_omni_demo_fixtures_green)"]
fn r1c_e_emit_gates_omni_suite_passes() {
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
