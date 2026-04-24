//! **Layer:** integration
//!
//! R1 T-TestGen gate `testgen_manual_claim_is_first_class` (ROADMAP.md:51, `[ext]`):
//! hand-authored `TestClaim` in `src/v3/std/r1_gates.dag` runs through the same
//! `TestRunner::run_suite` dispatch as generated claims (`test_runner.rs`).

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
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repo root")
        .to_path_buf();
    let gate = repo_root.join("src/v3/std/r1_gates.dag");
    let source =
        std::fs::read_to_string(&gate).unwrap_or_else(|err| panic!("read {gate:?}: {err}"));
    let dag = compile_clean(&source, "src/v3/std/r1_gates.dag");
    let results = TestRunner::new(&dag).run_suite("manual_claim_suite");

    assert_eq!(results.len(), 1, "expected one claim in manual_claim_suite");
    assert_eq!(results[0].claim_name, "testgen_manual_claim_is_first_class");
    assert_eq!(results[0].result, ClaimResult::Pass);
}
