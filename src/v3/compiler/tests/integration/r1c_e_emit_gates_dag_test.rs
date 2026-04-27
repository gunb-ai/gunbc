//! **Layer:** integration
//!
//! R1C-E — runner-side receipt that the T-Emit `.dag` `TestClaim` wrappers in
//! `tests/dag/r1c_e_emit_gates.template.dag` evaluate `Pass` through the same
//! `compile_to_dag` + `TestRunner` path as the rest of the `.dag` harness.
//!
//! The template carries `__R1C_E_BIN__` as the `ExecuteCommand` `command`
//! placeholder; the substitution to the real `bin` path happens here at
//! test-crate compile time (`env!("CARGO_BIN_EXE_r1c_e_emit_gates")`). No
//! checked-in absolute path goes stale (R1 Closure decision on #973 —
//! parallel to the `r1_gates.template.dag` splice discipline).
//!
//! Runs unignored: the only claim in the suite today (`generic-bounds`) is
//! structural-only — no `rustc` / `python3` / `go` toolchain dependency — so
//! the `.dag` wiring is exercised in normal CI. When `rust-green` / `omni-*`
//! subcommands land, those new claims may need toolchain gating; revisit at
//! that time (split into a separate `#[ignore]`d test if needed, rather than
//! re-gating this one).

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const TEMPLATE: &str = include_str!("../dag/r1c_e_emit_gates.template.dag");
const TEMPLATE_PATH: &str = "src/v3/compiler/tests/dag/r1c_e_emit_gates.template.dag";
const BIN_PATH: &str = env!("CARGO_BIN_EXE_r1c_e_emit_gates");
const BIN_PLACEHOLDER: &str = "__R1C_E_BIN__";

fn substituted_dag_source() -> String {
    assert!(
        TEMPLATE.contains(BIN_PLACEHOLDER),
        "template must contain `{BIN_PLACEHOLDER}` placeholder for bin substitution \
         (see manager guidance, #973): {TEMPLATE_PATH}"
    );
    TEMPLATE.replace(BIN_PLACEHOLDER, BIN_PATH)
}

#[test]
fn r1c_e_emit_gates_suite_passes_through_runner() {
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
