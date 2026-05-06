//! **Layer:** integration
//!
//! PB-Runtime ↔ R2-Evaluator Row-4 corpus seeds (1)–(2).
//! These tests wire the seed programs and TestClaim shape without making Row 4
//! green: the current runner must compile the seed sources, then return the
//! unsupported-producer `NotYetImplemented` receipt for
//! `(pb_runtime_evaluate, r2_evaluator_evaluate)`.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::test_runner::{ClaimResult, TestClaimValue, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r3_pb_runtime_evaluator_corpus_seeds.dag");
const FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/r3_pb_runtime_evaluator_corpus_seeds.dag";
const SUITE_NAME: &str = "r3_pb_runtime_evaluator_corpus_seed_suite";
const INT_CLAIM: &str = "pb_eval_corpus_seed_int_arithmetic_deferred";
const LIST_CLAIM: &str = "pb_eval_corpus_seed_list_fold_deferred";
const EXPECTED_CLAIMS: [&str; 2] = [INT_CLAIM, LIST_CLAIM];
const INT_SOURCE: &str = include_str!("../fixtures/r3_pb_eval_corpus/seed_int_arithmetic.v3");
const LIST_SOURCE: &str = include_str!("../fixtures/r3_pb_eval_corpus/seed_list_fold.v3");

fn lower(source: &'static str, file: &'static str) -> Dag {
    match compile_to_dag(source, file) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{file}: expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{file} should lower without module diagnostics. Got `Err(Semantic)`: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("unexpected compile error for {file}: {other:?}"),
    }
}

fn claim<'a>(dag: &'a Dag, claim_name: &str) -> TestClaimValue {
    let decl = dag
        .declaration_by_name(claim_name)
        .unwrap_or_else(|| panic!("missing `{claim_name}` in {FIXTURE_PATH}"));
    TestClaimValue::from_declaration(decl).unwrap_or_else(|reason| {
        panic!("`{claim_name}` should lower to a structural TestClaim: {reason}")
    })
}

#[test]
fn r3_pb_runtime_evaluator_corpus_seed_sources_compile_and_match_authority_files() {
    let fixture = lower(FIXTURE_SOURCE, FIXTURE_PATH);

    let int_claim = claim(&fixture, INT_CLAIM);
    assert_eq!(
        int_claim.source, INT_SOURCE,
        "`{INT_CLAIM}` source must match the seed file bytes"
    );
    lower(
        INT_SOURCE,
        "src/v3/compiler/tests/fixtures/r3_pb_eval_corpus/seed_int_arithmetic.v3",
    );

    let list_claim = claim(&fixture, LIST_CLAIM);
    assert_eq!(
        list_claim.source, LIST_SOURCE,
        "`{LIST_CLAIM}` source must match the seed file bytes"
    );
    lower(
        LIST_SOURCE,
        "src/v3/compiler/tests/fixtures/r3_pb_eval_corpus/seed_list_fold.v3",
    );
}

#[test]
fn r3_pb_runtime_evaluator_corpus_seed_suite_stays_deferred_until_row4_producers_land() {
    let fixture = lower(FIXTURE_SOURCE, FIXTURE_PATH);
    let results = TestRunner::new(&fixture).run_suite(SUITE_NAME);
    assert_eq!(results.len(), EXPECTED_CLAIMS.len());

    for expected_claim in EXPECTED_CLAIMS {
        let result = results
            .iter()
            .find(|result| result.claim_name == expected_claim)
            .unwrap_or_else(|| {
                panic!("missing `{expected_claim}` in `{SUITE_NAME}` results: {results:?}")
            });
        // Load-bearing Row-4 ratchet: the unsupported PB-Runtime / R2-Evaluator
        // producer pair must stay typed-NotYetImplemented until the real
        // producers land. Source byte-sync and independent seed compilation are
        // covered by `r3_pb_runtime_evaluator_corpus_seed_sources_compile_and_match_authority_files`.
        assert!(
            matches!(&result.result, ClaimResult::NotYetImplemented(_)),
            "{} should remain deferred on the Row-4 producer pair; got {:?}",
            result.claim_name,
            result.result
        );
    }
}
