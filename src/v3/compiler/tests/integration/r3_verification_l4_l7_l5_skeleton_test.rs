//! **Layer:** integration
//!
//! R3 Lane 1 + Lane 2 + L5 **implementation skeleton** pre-authoring: fixtures compile cleanly and
//! `TestRunner` receipts show intentionally deferred predicates (`NotYetImplemented`), not silent
//! structural failure. Matrix: `docs/briefs/r3-v-l7-algebra-coverage-matrix.md`.

use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::test_runner::{ClaimResult, TestClaimValue, TestRunner};
use v3_compiler::CompileError;

const L4_FIXTURE: &str = include_str!("../fixtures/r3_verification_l4_emit_eval_match.dag");
const L4_FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag";
const L4_SUITE: &str = "r3_verification_l4_emit_eval_skeleton_suite";
const L4_CLAIM: &str = "r3_verification_l4_emit_eval_match_skeleton";

const L7_FIXTURE: &str = include_str!("../fixtures/r3_verification_l7_algebraic_laws.dag");
const L7_FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/r3_verification_l7_algebraic_laws.dag";
const L7_SUITE: &str = "r3_verification_l7_algebra_skeleton_suite";
const L7_CLAIM: &str = "r3_verification_l7_algebraic_laws_skeleton";

const L5_FIXTURE: &str = include_str!("../fixtures/r3_verification_l5_corpus.dag");
const L5_FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag";
const L5_SUITE: &str = "r3_verification_l5_corpus_suite";
const L5_CLAIM: &str = "r3_verification_l5_cross_target_skeleton";
const L5_AUTHORITY_PROGRAM: &str = include_str!("../fixtures/r3_l5_corpus/add_then_branch_seed.v3");

static L4_DAG: OnceLock<Dag> = OnceLock::new();
static L7_DAG: OnceLock<Dag> = OnceLock::new();
static L5_DAG: OnceLock<Dag> = OnceLock::new();

fn cached_compile(
    src: &'static str,
    path: &'static str,
    cell: &'static OnceLock<Dag>,
) -> &'static Dag {
    cell.get_or_init(|| match compile_to_dag(src, path) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{path}: expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{path} should lower without module diagnostics. Got `Err(Semantic)`: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("unexpected compile error for {path}: {other:?}"),
    })
}

#[test]
fn r3_verification_l4_emit_eval_match_skeleton_is_nyi() {
    let dag = cached_compile(L4_FIXTURE, L4_FIXTURE_PATH, &L4_DAG);
    let results = TestRunner::new(dag).run_suite(L4_SUITE);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].claim_name, L4_CLAIM);
    assert!(
        matches!(results[0].result, ClaimResult::NotYetImplemented(_)),
        "expected deferred DifferentialEquals lineage for emit/eval pairing, got {:?}",
        results[0].result
    );
}

#[test]
fn r3_verification_l7_algebraic_law_identity_skeleton_is_nyi() {
    let dag = cached_compile(L7_FIXTURE, L7_FIXTURE_PATH, &L7_DAG);
    let results = TestRunner::new(dag).run_suite(L7_SUITE);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].claim_name, L7_CLAIM);
    assert!(
        matches!(results[0].result, ClaimResult::NotYetImplemented(_)),
        "expected AlgebraicLaw::Identity to stay deferred, got {:?}",
        results[0].result
    );
}

#[test]
fn r3_verification_l5_corpus_for_all_targets_skeleton_is_nyi() {
    let dag = cached_compile(L5_FIXTURE, L5_FIXTURE_PATH, &L5_DAG);
    let claim_decl = dag
        .declaration_by_name(L5_CLAIM)
        .unwrap_or_else(|| panic!("missing `{L5_CLAIM}` in {}", L5_FIXTURE_PATH));
    let claim = TestClaimValue::from_declaration(claim_decl).unwrap_or_else(|reason| {
        panic!("`{L5_CLAIM}` should lower to a structural TestClaim: {reason}");
    });
    assert_eq!(
        claim.source, L5_AUTHORITY_PROGRAM,
        "`TestClaim.source` must equal `add_then_branch_seed.v3` bytes (single program authority)"
    );
    let results = TestRunner::new(dag).run_suite(L5_SUITE);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].claim_name, L5_CLAIM);
    assert!(
        matches!(results[0].result, ClaimResult::NotYetImplemented(_)),
        "expected ForAllTargets default-runner deferral, got {:?}",
        results[0].result
    );
}
