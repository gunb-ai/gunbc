//! **Layer:** integration
//!
//! A.1.5a receipt: parse and wire-check the modeled in-process equivalence
//! claim for `TestClaimRun` over the fixed manual corpus slice.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full cross-module v4
//! execution is owned by the bootstrap evaluator harness lane.
//!
//! **P5 receipt (INVARIANTS.md §P5 Mechanism (b) — SG-0 `EXPECTED_HAND_AUTHORED_TEST`):**
//! explicit deferral to ROADMAP.md `T-PB-B` /
//! `pb_rust_tests_outside_residual_zero`; dissolves when the A.1 harness
//! executes this receipt as `.dag` TestClaim data.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceItem, SurfaceModule};
use v3_compiler::tokenize_for_test;

const EQUIVALENCE_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/testclaim_in_process_equivalence.dag");
const EQUIVALENCE_PATH: &str = "src/v4/test/claim/workflow/testclaim_in_process_equivalence.dag";
const BOOTSTRAP_DAG: &str = include_str!("../../../../v4/workflow/bootstrap.dag");
const BOOTSTRAP_PATH: &str = "src/v4/workflow/bootstrap.dag";

fn parse_module(source: &str, path: &str) -> SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn module_path(module: &SurfaceModule) -> Vec<&str> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn testclaim_in_process_equivalence_dag_tokenizes_and_parses() {
    let module = parse_module(EQUIVALENCE_DAG, EQUIVALENCE_PATH);
    assert_eq!(
        module_path(&module),
        vec![
            "v4",
            "test",
            "claim",
            "workflow",
            "testclaim_in_process_equivalence"
        ],
        "{EQUIVALENCE_PATH}: module path"
    );
}

#[test]
fn testclaim_in_process_equivalence_wires_fixed_slice_to_bootstrap_pin() {
    assert!(
        EQUIVALENCE_DAG.contains("type TestClaimRunInProcessEquivalenceReceipt")
            && EQUIVALENCE_DAG.contains("bootstrap_projection_inputs")
            && EQUIVALENCE_DAG.contains("bootstrap_projection_inputs_well_formed")
            && EQUIVALENCE_DAG.contains("run_manual_testclaim_corpus_eval()")
            && EQUIVALENCE_DAG.contains("receipt.in_process_report == receipt.harness_path_report")
            && EQUIVALENCE_DAG.contains("data witness_testclaim_run_in_process_equivalence: Bool"),
        "{EQUIVALENCE_PATH}: A.1.5a receipt must compare direct and bootstrap-harness reports"
    );
    assert!(
        BOOTSTRAP_DAG.contains("runtime_model: v4_evaluator_runtime_wave1()")
            && BOOTSTRAP_DAG.contains(
                "inputs.runtime_model == v4_evaluator_runtime_wave1()"
            ),
        "{BOOTSTRAP_PATH}: bootstrap projection inputs must remain pinned to v4_evaluator_runtime_wave1"
    );
}
