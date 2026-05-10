//! **Layer:** integration
//!
//! R3 T-Free-Consequences first-batch author-now/fire-later claims.
//! Gate `#43` asserts pairwise-independent top-level binds emit a parallel Rust schedule;
//! gate `#44` asserts dependent binds omit the parallel `thread::scope` emit path (sequential);
//! gate `#45` asserts a Bool branch lowers to `if … else` with no `thread::scope` scheduling on
//! the arms. Gate `#49` asserts repeated pure-call caching, and gate `#50` keeps the remaining
//! `BinaryDimensionReportEquals` author-now/fire-later shape.

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust;
use v3_compiler::test_runner::{ClaimResult, TestClaimValue, TestRunner};
use v3_compiler::CompileError;

use crate::common::run_on_larger_stack;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r3_free_consequences_first_batch.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_free_consequences_first_batch.dag";
const SUITE_NAME: &str = "r3_free_consequences_first_batch_suite";
const EXPECTED_CLAIMS: [&str; 5] = [
    "auto_parallelism_independent_binds_emit_parallel",
    "auto_parallelism_dependent_binds_emit_sequential",
    "auto_parallelism_branch_arms_serialize",
    "auto_memoization_repeated_pure_call_cached",
    "auto_memoization_no_caching_for_one_shot",
];

#[test]
fn r3_free_consequences_first_batch_reaches_unified_predicate_shape() {
    run_on_larger_stack(|| {
        r3_free_consequences_first_batch_reaches_unified_predicate_shape_inner()
    });
}

fn r3_free_consequences_first_batch_reaches_unified_predicate_shape_inner() {
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
    assert_eq!(results.len(), EXPECTED_CLAIMS.len());

    for (result, expected_name) in results.iter().zip(EXPECTED_CLAIMS) {
        assert_eq!(result.claim_name, expected_name);
        match expected_name {
            "auto_parallelism_independent_binds_emit_parallel"
            | "auto_parallelism_dependent_binds_emit_sequential" => {
                assert!(
                    matches!(&result.result, ClaimResult::Pass),
                    "expected {expected_name} (R3 gates #43/#44) to Pass, got {:?}",
                    result.result
                );
            }
            "auto_parallelism_branch_arms_serialize" => {
                assert!(
                    matches!(&result.result, ClaimResult::Pass),
                    "expected {expected_name} (R3 gate #45) to Pass, got {:?}",
                    result.result
                );
            }
            "auto_memoization_repeated_pure_call_cached" => {
                assert!(
                    matches!(&result.result, ClaimResult::Pass),
                    "expected {expected_name} (R3 gate #49) to Pass, got {:?}",
                    result.result
                );
                assert_repeated_pure_call_claim_emits_cached_target_code(&dag, expected_name);
            }
            _ => {
                assert!(
                    matches!(
                        &result.result,
                        ClaimResult::NotYetImplemented(reason)
                            if reason.contains("BinaryDimensionReportEquals")
                                && reason.contains("structural shape is valid")
                    ),
                    "expected {expected_name} to reach BinaryDimensionReportEquals deferred path, got {:?}",
                    result.result
                );
            }
        }
    }
}

fn assert_repeated_pure_call_claim_emits_cached_target_code(
    fixture_dag: &v3_compiler::dag::Dag,
    claim_name: &str,
) {
    let claim_decl = fixture_dag
        .declaration_by_name(claim_name)
        .unwrap_or_else(|| panic!("fixture must declare TestClaim `{claim_name}`"));
    let claim = TestClaimValue::from_declaration(claim_decl)
        .unwrap_or_else(|err| panic!("fixture TestClaim `{claim_name}` must parse: {err}"));
    let program_dag = compile_to_dag(&claim.source, &claim.file_name)
        .unwrap_or_else(|err| panic!("claim source `{}` must compile: {err:?}", claim.file_name));
    let emitted = emit_rust(&program_dag)
        .unwrap_or_else(|err| panic!("claim source `{}` must emit Rust: {err:?}", claim.file_name));

    assert_eq!(
        emitted.matches("expensive(&x)").count(),
        1,
        "gate #49 must emit the repeated pure call once, then reuse the cached bind; emitted:\n{emitted}"
    );
    assert!(
        emitted.contains("let first: i64 = expensive(&x);"),
        "gate #49 must materialize the first pure call bind; emitted:\n{emitted}"
    );
    assert!(
        emitted.contains("let second: i64 = first;"),
        "gate #49 must render the repeated pure call as a cached reuse of `first`; emitted:\n{emitted}"
    );
}
