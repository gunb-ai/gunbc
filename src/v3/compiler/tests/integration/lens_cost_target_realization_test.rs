//! T-CostLens-Composition Slice 1a.1 consumer-proof tests.
//!
//! Exercises `v3_compiler::lens_cost_target_realization::*` — the `.dag`-tier
//! consumer of the `declaration_by_name` substrate accessor introduced by
//! Slice 1a.0 (PR #2194 merged at commit 633f83854; Director ratification
//! at gunbc#828 #issuecomment-4402899692).
//!
//! Closes the same-PR-consumer-evidence gap per INVARIANTS P2 raised by
//! codex BLOCKING on PR #2194 sha 633f8385 (resolved post-merge by this
//! Slice 1a.1 landing).
//!
//! **R3 §1.8 gate #37** (`cost_lens_reads_target_realization`, ε path per
//! Q-Cost-Composition-Layering / PR #2181): demonstrates Rust-side reading of
//! (a) abstract `SymbolicCost` from `symbolic_cost_of` on a compiled program
//! and (b) the target language spec's `TypeRealization.cost` int field on a
//! bootstrap `rust_*` row, composed via `Semiring<SymbolicCost>` `sequential`.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    sequential, Behavior, Declaration, FieldValue, LiteralBits, SymbolicCost, ValueBody,
};
use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};
use v3_compiler::lens_cost_target_realization::{
    behavior_realization_meta, callable_realization_meta, operator_realization_meta,
    pattern_realization_meta, type_instantiation_realization_meta, type_realization_meta,
};

fn run_with_cost_target_realization_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("cost-target-realization-test".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn cost target realization test thread")
        .join()
        .expect("cost target realization test thread should not panic");
}

fn find_bind_value(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

/// `cost` field on a lowered `TypeRealization` / `CallableRealization` data row.
fn realization_row_cost_int(decl: &Declaration) -> i64 {
    let Some(body) = decl.value_body.as_ref() else {
        panic!("declaration {:?} missing value_body", decl.name);
    };
    let ValueBody::Structural { fields } = body else {
        panic!("expected structural realization row, got {body:?}");
    };
    for (key, value) in fields {
        if key == "cost" {
            let FieldValue::Literal(LiteralBits::Int(n)) = value else {
                panic!("`cost` must be Int literal, got {value:?}");
            };
            return *n;
        }
    }
    panic!("no `cost` field on realization row {:?}", decl.name);
}

#[test]
fn type_realization_meta_resolves_against_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    let meta = type_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "type_realization_meta should resolve `TypeRealization` declaration in bootstrap dag"
    );
    let decl = meta.unwrap();
    assert_eq!(
        decl.name.as_deref(),
        Some("TypeRealization"),
        "resolved declaration's name should be `TypeRealization`"
    );
}

#[test]
fn callable_realization_meta_resolves_against_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    let meta = callable_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "callable_realization_meta should resolve `CallableRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("CallableRealization"));
}

#[test]
fn operator_realization_meta_resolves_against_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    let meta = operator_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "operator_realization_meta should resolve `OperatorRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("OperatorRealization"));
}

#[test]
fn behavior_realization_meta_resolves_against_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    let meta = behavior_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "behavior_realization_meta should resolve `BehaviorRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("BehaviorRealization"));
}

#[test]
fn type_instantiation_realization_meta_resolves_against_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    let meta = type_instantiation_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "type_instantiation_realization_meta should resolve `TypeInstantiationRealization` in bootstrap"
    );
    assert_eq!(
        meta.unwrap().name.as_deref(),
        Some("TypeInstantiationRealization")
    );
}

#[test]
fn pattern_realization_meta_resolves_against_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    let meta = pattern_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "pattern_realization_meta should resolve `PatternRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("PatternRealization"));
}

/// R3 gate #37 — ε-path consumer: abstract cost × target `TypeRealization.cost`.
#[test]
fn cost_lens_composes_symbolic_cost_with_rust_type_realization_row() {
    run_with_cost_target_realization_stack(|| {
        let boot = generated_full_bootstrap_dag();
        let tr_meta = type_realization_meta(&boot).expect("TypeRealization meta in bootstrap");
        let rust_int = boot
            .declaration_by_name("rust_int")
            .expect("`rust_int` TypeRealization row from rust.dag");
        assert_eq!(
            rust_int.meta_tag,
            Some(tr_meta.id),
            "rust_int should carry TypeRealization meta_tag"
        );
        let target_primitive_cost = realization_row_cost_int(rust_int);
        assert_eq!(
            target_primitive_cost, 1,
            "fixture: rust_int.cost is 1 in src/v3/spec/rust.dag"
        );

        let user = compile_to_dag("let lit: Int = 7", "r3_gate37_cost_lens.v3")
            .expect("literal program compiles");
        let lit = find_bind_value(&user, "lit");
        let algebra_cost = match symbolic_cost_of(&user, &lit) {
            SymbolicCostLookup::Hit(c) => c,
            SymbolicCostLookup::Miss => panic!("symbolic_cost_of Miss for `lit`"),
        };
        assert!(
            matches!(algebra_cost, SymbolicCost::ConstantCost { _0: 0 }),
            "literal bind should stay constant zero at algebra layer, got {algebra_cost:?}"
        );

        let composed = sequential(
            algebra_cost,
            SymbolicCost::ConstantCost {
                _0: target_primitive_cost,
            },
        );
        assert!(
            matches!(composed, SymbolicCost::ConstantCost { _0: 1 }),
            "sequential(Constant(0), Constant(target_cost)) should normalize to Constant(1), got {composed:?}"
        );
    });
}

/// Same gate #37 surface on a `CallableRealization` row (list helper realization).
#[test]
fn cost_lens_reads_cost_field_on_rust_callable_realization_row() {
    let boot = generated_full_bootstrap_dag();
    let cr_meta = callable_realization_meta(&boot).expect("CallableRealization meta");
    let row = boot
        .declaration_by_name("rust_is_empty_callable")
        .expect("rust_is_empty_callable row");
    assert_eq!(row.meta_tag, Some(cr_meta.id));
    assert_eq!(realization_row_cost_int(row), 1);
}
