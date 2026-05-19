//! **Layer:** integration
//!
//! **T-15 receipt:** `compile_to_dag` on `src/v4/bin/main.dag` with empty diagnostics, plus
//! structural checks on the lowered `Dag` (named `data` rows + trampoline `String` payload).
//! Operator-groundedness **glyph / site / testcase form** is **HOLD** on this PR pending the
//! hub-relayed canonical spec (neat-hawk-87 cascade 2026-05-19); this harness pins compile +
//! lowered data shape only (`TESTING.md` — pin behavior, not comment layout).
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS table row land in the same PR.
//!
//! **Dissolution:** remove when `main.dag` obligations are exercised only by `.dag`
//! `TestClaim` rows / generated harness without this per-file Rust `compile_to_dag` probe.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{LiteralBits, ValueBody};
use v3_compiler::CompileError;

const MAIN_DAG: &str = include_str!("../../../../v4/bin/main.dag");
const MAIN_DAG_PATH: &str = "src/v4/bin/main.dag";

const TRAMPOLINE_RUST_LINE: &str = r#"include!("v4_main_generated.rs");"#;

#[test]
fn v4_bin_main_dag_compiles_with_trampoline_and_digest_stub_declarations() {
    let dag = match compile_to_dag(MAIN_DAG, MAIN_DAG_PATH) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "{MAIN_DAG_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{MAIN_DAG_PATH}: {other:?}"),
    };
    assert!(
        dag.diagnostics().is_empty(),
        "{MAIN_DAG_PATH}: expected empty diagnostics, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    for name in [
        "main_rs_trampoline_authority",
        "stub_stage1_emitted_rust_digest_placeholder",
        "stub_stage2_emitted_rust_digest_placeholder",
    ] {
        assert!(
            dag.declaration_by_name(name).is_some(),
            "{MAIN_DAG_PATH}: missing declaration {name:?}"
        );
    }

    let tramp = dag
        .declaration_by_name("main_rs_trampoline_authority")
        .expect("main_rs_trampoline_authority");
    match tramp.value_body.as_ref() {
        Some(ValueBody::Scalar(LiteralBits::String(s))) => assert_eq!(
            s.as_str(),
            TRAMPOLINE_RUST_LINE,
            "{MAIN_DAG_PATH}: trampoline String payload"
        ),
        other => panic!(
            "{MAIN_DAG_PATH}: expected lowered String scalar for trampoline authority, got {other:?}"
        ),
    }
}
