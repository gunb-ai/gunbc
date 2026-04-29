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

#[test]
fn r3_deferred_claim_rejects_untyped_marker_refs() {
    let source = r#"
import std.verification { R3DeferredClaim, TestClaim, TestSuite }

type R1GateMarker {}
type R3LaneMarker {}
type ReleaseAuthorityDoc {}

data arbitrary_gate: Int = 0
data r3_lane_marker: R3LaneMarker = {}
data release_authority_doc: ReleaseAuthorityDoc = {}

data bad_deferred_claim: TestClaim = {
  name: "bad_deferred",
  source: "let _: Int = 0",
  file_name: "bad_deferred.v3",
  predicate: R3DeferredClaim {
    deferred_gate: arbitrary_gate,
    r3_lane: r3_lane_marker,
    authority_doc: release_authority_doc
  },
  requires: []
}

data suite: TestSuite = {
  name: "bad_deferred_suite",
  claims: [bad_deferred_claim]
}
"#;
    let dag = compile_to_dag(source, "bad_r3_deferred_claim.dag")
        .expect("malformed deferral fixture still compiles structurally");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(
        matches!(
            &results[0].result,
            ClaimResult::Fail(reason)
                if reason.contains("deferred_gate")
                    && reason.contains("R1GateMarker")
                    && reason.contains("arbitrary_gate")
        ),
        "expected R3DeferredClaim to fail closed on untyped deferred_gate, got {results:?}"
    );
}
