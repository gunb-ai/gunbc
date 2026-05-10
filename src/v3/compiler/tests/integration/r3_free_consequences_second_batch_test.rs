//! **Layer:** integration
//!
//! R3 T-Free-Consequences second-batch author-now/fire-later claims. The auto-loop-parallelism
//! claims (#46–#48) assert `Pass` on a **staged** scalar only: `lane2_workflow` may be installed via
//! the magic-comment harness (`src/v3/compiler/src/r3_fc_lane2_loop_witness.rs`) and read by the
//! native `auto_loop_parallelism_pending_lens` path — see `ROADMAP.md` §"R3 second-batch auto-loop
//! scaffold". The directive lines are **author attestation**, not a compiler proof of iteration
//! independence or dependence; full `Lens<Iteration-Independence> * …` composition is out of scope
//! here. Gate **#48** additionally keeps a structural `std.list.fold` program in
//! `r3_free_consequences_auto_loop_parallelism_dependence.v3` so lowering still exercises a real
//! loop body; integration tests ratchet embedded `TestClaim.source` against that file byte-for-byte
//! and assert the claim program lowers to `Behavior::Loop` so the fold is exercised on the compile
//! path, not only carried as inert text. The first cross-target-optimization claim (#51) is
//! executable through the symbolic-cost lens: the host test proves a constant arithmetic subtree has
//! cost `1` before the folded literal target (`3`) is applied. Gate #52 executes a structural
//! cost comparison over claim-program operator transforms, the symbolic-cost lens, and Rust / Go /
//! Python `LanguageSpec` realization rows.

use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::dag::{ArithmeticOp, Behavior, OperatorKind, SymbolicCost};
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};
use v3_compiler::test_runner::{ClaimResult, TestClaimValue, TestRunner};
use v3_compiler::CompileError;

use crate::common::run_on_larger_stack;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r3_free_consequences_second_batch.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_free_consequences_second_batch.dag";
const SUITE_NAME: &str = "r3_free_consequences_second_batch_suite";
/// Byte-sync authority for gate #48 `TestClaim.source` (`include_str` ↔ embedded `.dag` string).
const GATE_48_PROGRAM_AUTHORITY: &str =
    include_str!("../fixtures/r3_free_consequences_auto_loop_parallelism_dependence.v3");
const EXPECTED_CLAIMS: [&str; 5] = [
    "auto_loop_parallelism_provable_independence_emits_parallel",
    "auto_loop_parallelism_unproven_falls_back_sequential",
    "auto_loop_parallelism_dependence_emits_sequential",
    "cross_target_optimization_constant_fold_consistent",
    "cross_target_optimization_cost_structurally_derived",
];

static SECOND_BATCH_DAG: OnceLock<Dag> = OnceLock::new();

fn second_batch_dag() -> &'static Dag {
    SECOND_BATCH_DAG.get_or_init(|| match compile_to_dag(FIXTURE_SOURCE, FIXTURE_PATH) {
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
    })
}

fn claim_by_name(name: &str) -> TestClaimValue {
    let dag = second_batch_dag();
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("{name} TestClaim declaration"));
    TestClaimValue::from_declaration(decl).unwrap_or_else(|_| panic!("{name} TestClaimValue"))
}

#[test]
fn r3_free_consequences_second_batch_reaches_expected_consumer_shapes() {
    run_on_larger_stack(|| {
        r3_free_consequences_second_batch_reaches_expected_consumer_shapes_inner()
    });
}

fn r3_free_consequences_second_batch_reaches_expected_consumer_shapes_inner() {
    let dag = second_batch_dag();
    let gate_48 = dag
        .declaration_by_name("auto_loop_parallelism_dependence_emits_sequential")
        .expect("gate #48 TestClaim declaration");
    let claim_48 = TestClaimValue::from_declaration(gate_48).expect("TestClaimValue");
    assert_eq!(
        claim_48.source,
        GATE_48_PROGRAM_AUTHORITY,
        "embedded `TestClaim.source` must match `r3_free_consequences_auto_loop_parallelism_dependence.v3` byte-for-byte"
    );

    let gate_48_program = match compile_to_dag(&claim_48.source, &claim_48.file_name) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "gate #48 claim program should compile without diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("gate #48 claim program compile error: {other:?}"),
    };
    assert!(
        gate_48_program
            .nodes()
            .iter()
            .any(|b| matches!(b, Behavior::Loop(_))),
        "gate #48 must lower `std.list.fold` to at least one Behavior::Loop (fold is not decorative)"
    );

    let results = TestRunner::new(dag).run_suite(SUITE_NAME);
    assert_eq!(results.len(), EXPECTED_CLAIMS.len());

    for (result, expected_name) in results.iter().zip(EXPECTED_CLAIMS) {
        assert_eq!(result.claim_name, expected_name);
        assert!(
            matches!(&result.result, ClaimResult::Pass),
            "expected {expected_name} to pass (loop witnesses use staged LensOutputEquals; gate #51 uses executable SymbolicCostExprEquals; gate #52 uses structural cost comparison), got {:?}",
            result.result
        );
    }
}

#[test]
fn cross_target_optimization_constant_fold_consistent_has_symbolic_cost_witness() {
    run_on_larger_stack(|| {
        let claim = claim_by_name("cross_target_optimization_constant_fold_consistent");
        let dag = compile_to_dag(&claim.source, &claim.file_name)
            .expect("gate #51 claim source compiles");
        let folded_bind = dag
            .nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .find(|bind| bind.name == "folded")
            .expect("gate #51 folded bind");
        let pre_fold_cost = match symbolic_cost_of(&dag, &folded_bind.value) {
            SymbolicCostLookup::Hit(cost) => cost,
            SymbolicCostLookup::Miss => panic!("gate #51 symbolic_cost_of returned Miss"),
        };
        assert!(
            matches!(pre_fold_cost, SymbolicCost::ConstantCost { _0: 1 }),
            "gate #51 pre-fold arithmetic subtree cost should be ConstantCost(1), got {pre_fold_cost:?}"
        );
        // This witness intentionally observes the pre-fold DAG; if lowering starts folding
        // `1 + 2` earlier, update this Add-shape check and the expected symbolic cost together.
        let transform = dag
            .nodes()
            .iter()
            .filter_map(Behavior::as_transform)
            .find(|t| {
                matches!(
                    &t.target,
                    v3_compiler::dag::TransformTarget::Operator(OperatorKind::Arithmetic(
                        ArithmeticOp::Add
                    ))
                )
            })
            .expect("gate #51 should lower `1 + 2` to an Add transform before folding");
        assert_eq!(
            transform.inputs.len(),
            2,
            "gate #51 Add transform should have two operands"
        );
    });
}
