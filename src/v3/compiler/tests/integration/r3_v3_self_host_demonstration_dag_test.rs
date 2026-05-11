//! **Layer:** boundary
//!
//! R3 gate #71 — `v3_self_host_demonstration` (T-V2-Retirement).
//!
//! [`compile_to_dag`] + [`TestRunner`] evaluates a [`TestClaim`] whose predicate is
//! [`ExecuteCommand`] over the `self_host_fixed_point` binary — the PB-Runtime bounded
//! host-spawn trampoline (`docs/r3-structure.md` demonstration principle).
//!
//! The `.dag` template carries `__SELF_HOST_FIXED_POINT_BIN__`; substitution uses
//! `env!("CARGO_BIN_EXE_self_host_fixed_point")` at integration-test compile time (R1
//! Closure `#973` discipline, parallel to `r1c_e_emit_gates_dag_test` / `r1c_e_emit_gates.template.dag`).
//!
//! **Toolchain:** the logical child runs DB-8’s staged ratchet and may invoke `rustc` when
//! `dsl/gunbc/compiler.dag` parses under v3 — requires a full Rust toolchain on the runner.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const TEMPLATE: &str = include_str!("../dag/r3_v3_self_host_demonstration.template.dag");
const TEMPLATE_PATH: &str = "src/v3/compiler/tests/dag/r3_v3_self_host_demonstration.template.dag";
const BIN_PATH: &str = env!("CARGO_BIN_EXE_self_host_fixed_point");
const BIN_PLACEHOLDER: &str = "__SELF_HOST_FIXED_POINT_BIN__";

fn substituted_dag_source() -> String {
    assert!(
        TEMPLATE.contains(BIN_PLACEHOLDER),
        "template must contain `{BIN_PLACEHOLDER}` placeholder for bin substitution \
         (see `r1c_e_emit_gates.template.dag` discipline): {TEMPLATE_PATH}"
    );
    TEMPLATE.replace(BIN_PLACEHOLDER, BIN_PATH)
}

#[test]
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
