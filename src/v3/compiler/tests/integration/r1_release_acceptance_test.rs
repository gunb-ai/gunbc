//! **Layer:** integration
//!
//! R1 release acceptance — strict PB gates 3/4 remain live runner predicates,
//! while Director-approved R3 deferrals are structural `DeclarationRef` edges
//! through `R3DeferredClaim` markers.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r1_release_acceptance.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r1_release_acceptance.dag";
const SUITE_NAME: &str = "r1_release_acceptance_suite";

const EXPECTED_CLAIM_NAMES: &[&str] = &[
    "pb_hand_rust_at_shim_floor",
    "lens_producer_files_remaining",
    "pb_self_compile_fixed_point",
    "pb_compiler_std_ratchet_zero",
    "pb_test_file_generated_from_dag",
    "pb_rust_tests_outside_residual_zero",
];

#[test]
fn r1_release_acceptance_suite_passes_at_head() {
    let dag = match compile_to_dag(FIXTURE_SOURCE, FIXTURE_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{FIXTURE_PATH}: expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{FIXTURE_PATH} should lower without module diagnostics. Got `Err(Semantic)`: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("unexpected compile error for {FIXTURE_PATH}: {other:?}"),
    };

    let results = TestRunner::new(&dag).run_suite(SUITE_NAME);
    let actual_names: Vec<&str> = results.iter().map(|r| r.claim_name.as_str()).collect();
    assert_eq!(
        actual_names, EXPECTED_CLAIM_NAMES,
        "suite `{SUITE_NAME}`: claim order must match the release gate list"
    );

    let unimplemented: Vec<_> = results
        .iter()
        .filter(|r| matches!(r.result, ClaimResult::NotYetImplemented(_)))
        .collect();
    assert!(
        unimplemented.is_empty(),
        "release acceptance predicates must be wired, not `NotYetImplemented`. Offenders:\n{unimplemented:#?}"
    );

    for result in &results {
        assert_eq!(
            result.result,
            ClaimResult::Pass,
            "release acceptance claim `{}` should pass at HEAD",
            result.claim_name
        );
    }
}
