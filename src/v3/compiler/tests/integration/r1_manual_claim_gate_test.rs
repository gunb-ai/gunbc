//! **Layer:** integration
//!
//! R1 T-TestGen gate `testgen_manual_claim_is_first_class` (ROADMAP.md:51, `[ext]`):
//! hand-authored `TestClaim` in `tests/fixtures/r1_manual_claim_gate.dag` runs through the same
//! `TestRunner::run_suite` dispatch as generated claims (`test_runner.rs`).
//!
//! The fixture lives outside `src/v3/std/` so `regen_bootstrap` does not merge it into
//! `Dag::new()` twice (re-parsing `src/v3/std/*.dag` on top of the embedded snapshot would
//! duplicate declarations).

use std::path::PathBuf;

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

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

#[test]
fn testgen_manual_claim_is_first_class_gate_passes() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gate = manifest_dir.join("tests/fixtures/r1_manual_claim_gate.dag");
    let source =
        std::fs::read_to_string(&gate).unwrap_or_else(|err| panic!("read {gate:?}: {err}"));
    let dag = compile_clean(
        &source,
        "src/v3/compiler/tests/fixtures/r1_manual_claim_gate.dag",
    );
    let results = TestRunner::new(&dag).run_suite("manual_claim_suite");

    assert_eq!(results.len(), 1, "expected one claim in manual_claim_suite");
    assert_eq!(results[0].claim_name, "testgen_manual_claim_is_first_class");
    assert_eq!(results[0].result, ClaimResult::Pass);
}
