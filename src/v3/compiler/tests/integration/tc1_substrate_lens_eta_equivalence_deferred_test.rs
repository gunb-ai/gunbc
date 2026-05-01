//! **Layer:** integration
//!
//! TC1 substrate lens eta-equivalence — strict consumer shape uses
//! `BinaryDimensionReportEquals`; `SubstrateResearchDeferredClaim` remains
//! fixture-scoped and must not widen into general runner validity.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str =
    include_str!("../fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag");
const FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag";
const SUITE_NAME: &str = "tc1_substrate_lens_eta_equivalence_suite";

#[test]
fn tc1_substrate_lens_eta_suite_reaches_unified_predicate_shape() {
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
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].claim_name,
        "eta_equivalent_dag_forms_yield_identical_lens_results"
    );
    assert!(
        matches!(
            &results[0].result,
            ClaimResult::NotYetImplemented(reason)
                if reason.contains("BinaryDimensionReportEquals")
                    && reason.contains("structural shape is valid")
        ),
        "expected BinaryDimensionReportEquals shape-valid deferred result, got {:?}",
        results[0].result
    );
}

#[test]
fn substrate_research_deferred_claim_rejects_well_typed_refs_outside_tc1_fixture() {
    let source = r##"
import std.verification { SubstrateResearchDeferredClaim, TestClaim, TestSuite }

type Tc1ResearchGateMarker {}
type SubstrateLensPrimitiveTargetLaneMarker {}
type LambdaCalculusGroundingAuthorityDoc {}

data gate_marker: Tc1ResearchGateMarker = {}
data target_lane_marker: SubstrateLensPrimitiveTargetLaneMarker = {}
data authority_doc: LambdaCalculusGroundingAuthorityDoc = {}

data bad_deferred_claim: TestClaim = {
  name: "bad_deferred",
  source: "let _: Int = 0",
  file_name: "bad_deferred.v3",
  predicate: SubstrateResearchDeferredClaim {
    deferred_gate: gate_marker,
    target_lane: target_lane_marker,
    authority_doc: authority_doc
  },
  requires: []
}

data suite: TestSuite = {
  name: "bad_suite",
  claims: [bad_deferred_claim]
}
"##;
    let dag = compile_to_dag(source, "arbitrary_substrate_research_deferred_claim.dag")
        .expect("well-typed deferral fixture compiles structurally");
    let results = TestRunner::new(&dag).run_suite("suite");

    assert_eq!(results.len(), 1);
    assert!(
        matches!(&results[0].result, ClaimResult::Fail(_)),
        "expected SubstrateResearchDeferredClaim to fail outside TC1 fixture, got {results:?}"
    );
}
