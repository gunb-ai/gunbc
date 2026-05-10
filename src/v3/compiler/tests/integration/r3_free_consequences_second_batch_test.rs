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
//! path, not only carried as inert text. Gate #51 still locks the cost-related
//! `BinaryDimensionReportEquals` shape and stays `NotYetImplemented`; gate #52 adds a host-side
//! executable receipt that composes `Lens<SymbolicCost>` with `LanguageSpec` realization rows.

use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    sequential, ArithmeticOp, Behavior, ComparisonOp, Dag, DeclarationId, OperatorKind,
    SymbolicCost, TransformTarget,
};
use v3_compiler::emit_rust::emit_rust;
use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};
use v3_compiler::realization_cost::{RealizationCostKey, RealizationCostTable};
use v3_compiler::test_runner::{ClaimResult, TestClaimValue, TestRunner};
use v3_compiler::CompileError;

use crate::common::run_on_larger_stack;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r3_free_consequences_second_batch.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_free_consequences_second_batch.dag";
const SUITE_NAME: &str = "r3_free_consequences_second_batch_suite";
const GATE_52_PROGRAM_FILE: &str = "r3_free_consequences_gate52_cost_structurally_derived.v3";
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

    for (idx, (result, expected_name)) in results.iter().zip(EXPECTED_CLAIMS).enumerate() {
        assert_eq!(result.claim_name, expected_name);
        if idx < 3 {
            assert!(
                matches!(&result.result, ClaimResult::Pass),
                "expected {expected_name} to pass (LensOutputEquals matches staged loop-parallelism indicator), got {:?}",
                result.result
            );
        } else if idx == 3 {
            assert!(
                matches!(&result.result, ClaimResult::NotYetImplemented(_)),
                "expected {expected_name} to stay author-now/fire-later on BinaryDimensionReportEquals, got {:?}",
                result.result
            );
        } else {
            assert!(
                matches!(&result.result, ClaimResult::NotYetImplemented(_)),
                "expected {expected_name} `.dag` predicate to stay at the generic BinaryDimensionReportEquals boundary; \
                 gate #52's executable receipt lives in `cross_target_optimization_cost_structurally_derived_receipt`, got {:?}",
                result.result
            );
        }
    }
}

#[test]
fn cross_target_optimization_cost_structurally_derived_receipt() {
    run_on_larger_stack(|| {
        let boot = generated_full_bootstrap_dag();
        let rust_language = named_id(&boot, "rust_language");
        let int_decl = named_id(&boot, "Int");
        let table = RealizationCostTable::for_language(&boot, rust_language)
            .expect("Rust LanguageSpec realization-cost table should build");

        let user = compile_to_dag(
            "\
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)

let demo: Int = countdown(3) + 1
",
            GATE_52_PROGRAM_FILE,
        )
        .expect("gate #52 representative program compiles");
        let emitted = emit_rust(&user).expect("gate #52 representative program emits to Rust");
        assert!(
            emitted.contains("fn countdown")
                && emitted.contains("countdown(&(((*(p0)) - 1)))")
                && emitted.contains("let demo: i64 = (countdown(&(3)) + 1);"),
            "gate #52 must exercise the emitted target program, got:\n{emitted}"
        );

        let countdown = find_bind_value(&user, "countdown");
        let algebra_cost = match symbolic_cost_of(&user, &countdown) {
            SymbolicCostLookup::Hit(cost) => cost,
            SymbolicCostLookup::Miss => panic!("symbolic_cost_of Miss for `countdown`"),
        };

        let realized_costs = realized_primitive_costs_from_program(&boot, &table, &user, int_decl);
        assert_eq!(
            realized_costs,
            vec![1, 1, 1, 1],
            "gate #52 fixture should derive Add/Add/Eq/Sub primitive costs from Rust LanguageSpec rows"
        );

        let observed_target_cost =
            realized_costs
                .iter()
                .copied()
                .fold(algebra_cost.clone(), |acc, primitive_cost| {
                    sequential(acc, SymbolicCost::ConstantCost { _0: primitive_cost })
                });
        let expected_structural_cost =
            compose_expected_structural_cost(algebra_cost, &boot, &table, int_decl);

        assert_eq!(
            observed_target_cost, expected_structural_cost,
            "gate #52 target-cost reading must equal Lens<SymbolicCost> composed with LanguageSpec realization costs"
        );
        assert!(
            mentions_linear(&observed_target_cost),
            "gate #52 receipt should preserve the recursive program's structural linear bound, got {observed_target_cost:?}"
        );
    });
}

fn realized_primitive_costs_from_program(
    boot: &Dag,
    table: &RealizationCostTable,
    program: &Dag,
    int_decl: DeclarationId,
) -> Vec<i64> {
    let mut costs = Vec::new();
    for transform in program
        .nodes()
        .iter()
        .filter_map(Behavior::as_transform)
        .filter(|transform| transform.span.file == GATE_52_PROGRAM_FILE)
    {
        let Some(op_row) = operator_row_for_transform(boot, &transform.target, int_decl) else {
            continue;
        };
        let cost = table
            .cost(&RealizationCostKey::Operator {
                target: int_decl,
                op: op_row,
            })
            .unwrap_or_else(|| {
                panic!("missing Rust LanguageSpec realization cost for operator row {op_row:?}")
            })
            .value();
        costs.push(cost);
    }
    costs.sort_unstable();
    costs
}

fn compose_expected_structural_cost(
    algebra_cost: SymbolicCost,
    boot: &Dag,
    table: &RealizationCostTable,
    int_decl: DeclarationId,
) -> SymbolicCost {
    // Mirrors the gate #52 fixture body exactly: one recursive decrement (`n - 1`),
    // one branch equality (`n == 0`), and two additions (recursive body plus `demo + 1`).
    [
        "rust_int_add",
        "rust_int_add",
        "rust_int_eq",
        "rust_int_sub",
    ]
    .into_iter()
    .map(|decl_name| {
        let op = field_ref(boot, decl_name, "op");
        table
            .cost(&RealizationCostKey::Operator {
                target: int_decl,
                op,
            })
            .unwrap_or_else(|| panic!("missing Rust LanguageSpec cost for `{decl_name}`"))
            .value()
    })
    .fold(algebra_cost, |acc, primitive_cost| {
        sequential(acc, SymbolicCost::ConstantCost { _0: primitive_cost })
    })
}

fn operator_row_for_transform(
    boot: &Dag,
    target: &TransformTarget,
    int_decl: DeclarationId,
) -> Option<DeclarationId> {
    let row = match target {
        TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)) => "rust_int_add",
        TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Sub)) => "rust_int_sub",
        TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)) => "rust_int_eq",
        _ => return None,
    };
    let row_decl = boot
        .declaration_by_name(row)
        .unwrap_or_else(|| panic!("missing `{row}` OperatorRealization row"));
    assert_eq!(
        field_ref(boot, row, "target"),
        int_decl,
        "`{row}` should realize the bootstrap Int target"
    );
    Some(
        row_decl
            .value_body
            .as_ref()
            .map(|_| field_ref(boot, row, "op"))
            .expect("OperatorRealization row should have a structural body"),
    )
}

fn find_bind_value(dag: &Dag, name: &str) -> v3_compiler::dag::PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn named_id(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("missing declaration `{name}`"))
        .id
}

fn field_ref(dag: &Dag, decl_name: &str, field_name: &str) -> DeclarationId {
    let decl = dag
        .declaration_by_name(decl_name)
        .unwrap_or_else(|| panic!("missing declaration `{decl_name}`"));
    let Some(v3_compiler::dag::ValueBody::Structural { fields }) = decl.value_body.as_ref() else {
        panic!("`{decl_name}` should have a structural body");
    };
    fields
        .iter()
        .find_map(|(key, value)| {
            (key == field_name).then_some(match value {
                v3_compiler::dag::FieldValue::Reference(id) => *id,
                other => {
                    panic!("`{decl_name}.{field_name}` should be DeclarationRef, got {other:?}")
                }
            })
        })
        .unwrap_or_else(|| panic!("`{decl_name}` missing field `{field_name}`"))
}

fn mentions_linear(cost: &SymbolicCost) -> bool {
    match cost {
        SymbolicCost::LinearCost { .. } => true,
        SymbolicCost::SumCost { _0: terms } | SymbolicCost::ProductCost { _0: terms } => {
            terms.iter().any(|term| mentions_linear(term.as_ref()))
        }
        _ => false,
    }
}
