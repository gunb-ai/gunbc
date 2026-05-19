//! **Layer:** integration
//!
//! **T-15 / RULING-1 groundedness receipt:** `compile_to_dag` cleanliness for
//! `src/v4/bin/main.dag` plus a source ratchet on the trampoline literal and the
//! per-carrier `// 🟢` / `// 🟡 groundedness (RULING-1)` one-liners (operator-direct
//! standing; header prose remains under #3358-A2 carve-out).
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS table row land in the same PR.
//!
//! **Dissolution:** remove when `main.dag` obligations are exercised only by `.dag`
//! `TestClaim` rows / generated harness without this per-file Rust `compile_to_dag` probe.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const MAIN_DAG: &str = include_str!("../../../../v4/bin/main.dag");
const MAIN_DAG_PATH: &str = "src/v4/bin/main.dag";

#[test]
fn v4_bin_main_dag_compiles_and_carries_groundedness_one_liners() {
    assert!(
        MAIN_DAG.contains("main_rs_trampoline_authority"),
        "{MAIN_DAG_PATH}: expected trampoline data id"
    );
    assert!(
        MAIN_DAG.contains(r#"include!("v4_main_generated.rs");"#),
        "{MAIN_DAG_PATH}: expected trampoline include! literal"
    );
    for needle in [
        "// 🟢 groundedness (RULING-1) — pretty much done:",
        "// 🟡 groundedness (RULING-1) — needs more work:",
    ] {
        assert!(
            MAIN_DAG.contains(needle),
            "{MAIN_DAG_PATH}: missing groundedness one-liner: {needle:?}"
        );
    }

    match compile_to_dag(MAIN_DAG, MAIN_DAG_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{MAIN_DAG_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{MAIN_DAG_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{MAIN_DAG_PATH}: {other:?}"),
    }
}
