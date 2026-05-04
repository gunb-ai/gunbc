//! **Layer:** integration
//!
//! R3 Lane 1 + Lane 2 + L5 harness receipts: Lane 1 L4 now exercises the wired W1
//! `DifferentialEquals(rust_emit_output, dag_eval_output)` path (plus a mixed-lineage
//! `NotYetImplemented` control), and Lane 1 L7 now exercises the wired `Associativity`
//! and `Commutativity` operational witnesses. Lane 2 / L5 rows remain intentionally deferred
//! where noted.
//! Matrix: `docs/briefs/r3-v-l7-algebra-coverage-matrix.md`.

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
const L4_FALSE_CLAIM: &str = "r3_verification_l4_emit_eval_false_branch";
const L4_NESTED_CLAIM: &str = "r3_verification_l4_emit_eval_nested_branch";

const L4_MIXED_FIXTURE: &str =
    include_str!("../fixtures/r3_verification_l4_emit_eval_mixed_lineage.dag");
const L4_MIXED_FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_mixed_lineage.dag";
const L4_MIXED_SUITE: &str = "r3_verification_l4_emit_eval_mixed_lineage_suite";
const L4_MIXED_CLAIM: &str = "r3_verification_l4_emit_eval_mixed_lineage_claim";

const L7_FIXTURE: &str = include_str!("../fixtures/r3_verification_l7_algebraic_laws.dag");
const L7_FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/r3_verification_l7_algebraic_laws.dag";
const L7_SUITE: &str = "r3_verification_l7_algebra_skeleton_suite";
const L7_CLAIM: &str = "r3_verification_l7_algebraic_laws_skeleton";
const L7_SEMIGROUP_ASSOCIATIVITY_CLAIM: &str = "r3_l7_semigroup_associativity";
const L7_COMMUTATIVE_MONOID_COMMUTATIVITY_CLAIM: &str = "r3_l7_commutative_monoid_commutativity";
const L7_MATRIX_SUITE: &str = "r3_verification_l7_algebra_matrix_suite";
const L7_MATRIX_WIRED_COUNT: usize = 8;
const L7_MATRIX_IDENTITY_NYI_COUNT: usize = 8;

const L5_FIXTURE: &str = include_str!("../fixtures/r3_verification_l5_corpus.dag");
const L5_FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag";
const L5_SUITE: &str = "r3_verification_l5_corpus_suite";
const L5_CLAIM: &str = "r3_verification_l5_cross_target_skeleton";
const L5_AUTHORITY_PROGRAM: &str = include_str!("../fixtures/r3_l5_corpus/add_then_branch_seed.v3");

static L4_DAG: OnceLock<Dag> = OnceLock::new();
static L4_MIXED_DAG: OnceLock<Dag> = OnceLock::new();
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

fn run_on_larger_stack<T>(f: impl FnOnce() -> T + Send + 'static) -> T
where
    T: Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(f)
        .expect("spawn larger-stack integration thread")
        .join()
        .expect("larger-stack integration thread panicked")
}

#[test]
fn r3_verification_l4_emit_eval_match_skeleton_passes_w1_emit_vs_eval() {
    run_on_larger_stack(|| {
        let dag = cached_compile(L4_FIXTURE, L4_FIXTURE_PATH, &L4_DAG);
        let results = TestRunner::new(dag).run_suite(L4_SUITE);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].claim_name, L4_CLAIM);
        assert!(
            matches!(results[0].result, ClaimResult::Pass),
            "expected W1 DifferentialEquals(rust_emit_output, dag_eval_output) Pass (branch literal 3); got {:?}",
            results[0].result
        );
    });
}

#[test]
fn r3_verification_l4_emit_eval_false_branch_passes_w1_emit_vs_eval() {
    run_on_larger_stack(|| {
        let dag = cached_compile(L4_FIXTURE, L4_FIXTURE_PATH, &L4_DAG);
        let results = TestRunner::new(dag).run_suite(L4_SUITE);
        assert_eq!(results.len(), 3);
        assert_eq!(results[1].claim_name, L4_FALSE_CLAIM);
        assert!(
            matches!(results[1].result, ClaimResult::Pass),
            "expected W1 DifferentialEquals(rust_emit_output, dag_eval_output) Pass (false branch signed Int -4); got {:?}",
            results[1].result
        );
    });
}

#[test]
fn r3_verification_l4_emit_eval_nested_branch_passes_w1_emit_vs_eval() {
    run_on_larger_stack(|| {
        let dag = cached_compile(L4_FIXTURE, L4_FIXTURE_PATH, &L4_DAG);
        let results = TestRunner::new(dag).run_suite(L4_SUITE);
        assert_eq!(results.len(), 3);
        assert_eq!(results[2].claim_name, L4_NESTED_CLAIM);
        assert!(
            matches!(results[2].result, ClaimResult::Pass),
            "expected W1 DifferentialEquals(rust_emit_output, dag_eval_output) Pass (nested branch Int 7); got {:?}",
            results[2].result
        );
    });
}

#[test]
fn r3_verification_l4_emit_eval_mixed_lineage_stays_not_yet_implemented() {
    run_on_larger_stack(|| {
        let dag = cached_compile(L4_MIXED_FIXTURE, L4_MIXED_FIXTURE_PATH, &L4_MIXED_DAG);
        let results = TestRunner::new(dag).run_suite(L4_MIXED_SUITE);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].claim_name, L4_MIXED_CLAIM);
        let ClaimResult::NotYetImplemented(msg) = &results[0].result else {
            panic!(
                "expected mixed (rust_emit_output, v3_program_cost) pairing to stay deferred, got {:?}",
                results[0].result
            );
        };
        assert!(
            msg.contains("unsupported producer pairing"),
            "NYI receipt should name unsupported producer pairing (producer-identity gate); got {msg}"
        );
        assert!(
            msg.contains("#1495"),
            "NYI receipt should cite #1495 rebase / ratchet coordination; got {msg}"
        );
    });
}

#[test]
fn r3_verification_l7_semigroup_associativity_passes_wired_associativity() {
    run_on_larger_stack(|| {
        let dag = cached_compile(L7_FIXTURE, L7_FIXTURE_PATH, &L7_DAG);
        let results = TestRunner::new(dag).run_suite(L7_MATRIX_SUITE);
        let result = results
            .iter()
            .find(|result| result.claim_name == L7_SEMIGROUP_ASSOCIATIVITY_CLAIM)
            .unwrap_or_else(|| {
                panic!(
                    "missing `{}` in {}",
                    L7_SEMIGROUP_ASSOCIATIVITY_CLAIM, L7_FIXTURE_PATH
                )
            });
        assert!(
            matches!(result.result, ClaimResult::Pass),
            "expected L7 Associativity wire-up to pass for `{}`; got {:?}",
            L7_SEMIGROUP_ASSOCIATIVITY_CLAIM,
            result.result
        );
    });
}

#[test]
fn r3_verification_l7_commutative_monoid_commutativity_passes_wired_commutativity() {
    run_on_larger_stack(|| {
        let dag = cached_compile(L7_FIXTURE, L7_FIXTURE_PATH, &L7_DAG);
        let results = TestRunner::new(dag).run_suite(L7_MATRIX_SUITE);
        let result = results
            .iter()
            .find(|result| result.claim_name == L7_COMMUTATIVE_MONOID_COMMUTATIVITY_CLAIM)
            .unwrap_or_else(|| {
                panic!(
                    "missing `{}` in {}",
                    L7_COMMUTATIVE_MONOID_COMMUTATIVITY_CLAIM, L7_FIXTURE_PATH
                )
            });
        assert!(
            matches!(result.result, ClaimResult::Pass),
            "expected L7 Commutativity wire-up to pass for `{}`; got {:?}",
            L7_COMMUTATIVE_MONOID_COMMUTATIVITY_CLAIM,
            result.result
        );
    });
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
fn r3_verification_l7_algebraic_law_matrix_has_current_runner_receipts() {
    let dag = cached_compile(L7_FIXTURE, L7_FIXTURE_PATH, &L7_DAG);
    let results = TestRunner::new(dag).run_suite(L7_MATRIX_SUITE);
    assert_eq!(
        results.len(),
        L7_MATRIX_WIRED_COUNT + L7_MATRIX_IDENTITY_NYI_COUNT
    );

    let mut wired_passes = 0;
    let mut identity_deferred = 0;
    for result in &results {
        if result.claim_name.ends_with("_identity") {
            assert!(
                matches!(result.result, ClaimResult::NotYetImplemented(_)),
                "{} should remain deferred until identity-element substrate edges land; got {:?}",
                result.claim_name,
                result.result
            );
            identity_deferred += 1;
        } else {
            assert!(
                matches!(result.result, ClaimResult::Pass),
                "{} should exercise the current wired AlgebraicLaw witness path; got {:?}",
                result.claim_name,
                result.result
            );
            wired_passes += 1;
        }
    }
    assert_eq!(wired_passes, L7_MATRIX_WIRED_COUNT);
    assert_eq!(identity_deferred, L7_MATRIX_IDENTITY_NYI_COUNT);
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
