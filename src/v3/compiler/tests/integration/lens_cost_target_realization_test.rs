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
//! Q-Cost-Composition-Layering / PR #2181): **partial integration receipt**
//! (`docs/r3-program-plan.md` — not full emit-time `LanguageSpec` consumer;
//! that remains gates **#40** / **#70**). Proves (i) **TypeRealization**:
//! `symbolic_cost_of` × `rust_int` row `cost` via `sequential`; (ii)
//! **CallableRealization**: `rust_is_empty_callable` row `cost` readable from
//! the same lowered structural shape (**no** extra `sequential` pin in that
//! subtest — see program-plan gate row).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    literal_decimal_i64, per_call_descent_evidence, sequential, ArithmeticOp, Behavior,
    ComparisonOp, Dag, Declaration, DeclarationId, FieldValue, LiteralBits, OperatorKind,
    SymbolicCost, TransformTarget, ValueBody,
};
use v3_compiler::emit_rust::emit_rust;
use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};
use v3_compiler::lens_cost_target_realization::{
    behavior_realization_meta, callable_realization_meta, operator_realization_meta,
    pattern_realization_meta, type_instantiation_realization_meta, type_realization_meta,
};
use v3_compiler::realization_cost::{
    RealizationCostCategory, RealizationCostKey, RealizationCostTable,
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
            return literal_decimal_i64(n.as_str()).unwrap_or_else(|| {
                panic!("`cost` Int literal must be signed decimal i64, got {n:?}");
            });
        }
    }
    panic!("no `cost` field on realization row {:?}", decl.name);
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

#[test]
fn type_realization_meta_resolves_against_bootstrap() {
    let dag = bootstrap_dag();
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
    let dag = bootstrap_dag();
    let meta = callable_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "callable_realization_meta should resolve `CallableRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("CallableRealization"));
}

#[test]
fn operator_realization_meta_resolves_against_bootstrap() {
    let dag = bootstrap_dag();
    let meta = operator_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "operator_realization_meta should resolve `OperatorRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("OperatorRealization"));
}

#[test]
fn behavior_realization_meta_resolves_against_bootstrap() {
    let dag = bootstrap_dag();
    let meta = behavior_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "behavior_realization_meta should resolve `BehaviorRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("BehaviorRealization"));
}

#[test]
fn type_instantiation_realization_meta_resolves_against_bootstrap() {
    let dag = bootstrap_dag();
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
    let dag = bootstrap_dag();
    let meta = pattern_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "pattern_realization_meta should resolve `PatternRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("PatternRealization"));
}

#[test]
fn realization_cost_table_reads_rust_type_realization_cost() {
    let dag = bootstrap_dag();
    let rust_language = named_id(&dag, "rust_language");
    let int_decl = named_id(&dag, "Int");

    let table = RealizationCostTable::for_language(&dag, rust_language)
        .expect("rust realization-cost table should build from structural rows");
    let entry = table
        .get(&RealizationCostKey::Type(int_decl))
        .expect("rust Int TypeRealization cost should be indexed by target declaration");

    assert_eq!(entry.language, rust_language);
    assert_eq!(entry.category(), RealizationCostCategory::Type);
    assert_eq!(entry.cost.value(), 1);
    assert_eq!(entry.declaration, named_id(&dag, "rust_int"));
}

#[test]
fn realization_cost_table_reads_zero_cost_behavior_realization() {
    let dag = bootstrap_dag();
    let rust_language = named_id(&dag, "rust_language");
    let let_stmt_target = field_ref(&dag, "rust_let_stmt", "target");

    let table = RealizationCostTable::for_language(&dag, rust_language)
        .expect("rust realization-cost table should build from structural rows");

    assert_eq!(
        table
            .cost(&RealizationCostKey::Behavior(let_stmt_target))
            .map(|cost| cost.value()),
        Some(0),
        "BehaviorRealization.cost must be observable, including zero-cost rows"
    );
}

#[test]
fn realization_cost_table_indexes_operator_by_target_and_op() {
    let dag = bootstrap_dag();
    let rust_language = named_id(&dag, "rust_language");
    let target = field_ref(&dag, "rust_int_add", "target");
    let op = field_ref(&dag, "rust_int_add", "op");

    let table = RealizationCostTable::for_language(&dag, rust_language)
        .expect("rust realization-cost table should build from structural rows");
    let entry = table
        .get(&RealizationCostKey::Operator { target, op })
        .expect("rust int add OperatorRealization cost should use (target, op) key");

    assert_eq!(entry.category(), RealizationCostCategory::Operator);
    assert_eq!(entry.cost.value(), 1);
    assert_eq!(entry.declaration, named_id(&dag, "rust_int_add"));
}

#[test]
fn realization_cost_table_filters_by_language() {
    let dag = bootstrap_dag();
    let rust_language = named_id(&dag, "rust_language");
    let go_language = named_id(&dag, "go_language");
    let int_decl = named_id(&dag, "Int");

    let rust_table = RealizationCostTable::for_language(&dag, rust_language)
        .expect("rust realization-cost table should build");
    let go_table =
        RealizationCostTable::for_language(&dag, go_language).expect("go table should build");

    assert_eq!(
        rust_table
            .get(&RealizationCostKey::Type(int_decl))
            .map(|entry| entry.declaration),
        Some(named_id(&dag, "rust_int"))
    );
    assert_eq!(
        go_table
            .get(&RealizationCostKey::Type(int_decl))
            .map(|entry| entry.declaration),
        Some(named_id(&dag, "go_int"))
    );
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

/// R3 gate #70 — representative target-program receipt.
///
/// This exercises the same structural composition the lane promises end-to-end:
/// compile a source program, emit a Rust target program, read algebra-level
/// cost from the symbolic-cost lens, read target primitive costs from the Rust
/// `LanguageSpec` realization table, then compose the two with the
/// `SymbolicCost` sequential algebra.
#[test]
fn cost_lens_demonstration_composes_representative_rust_program_cost() {
    run_with_cost_target_realization_stack(|| {
        let boot = generated_full_bootstrap_dag();
        let rust_language = named_id(&boot, "rust_language");
        let int_decl = named_id(&boot, "Int");
        let add_op = field_ref(&boot, "rust_int_add", "op");
        let sub_op = field_ref(&boot, "rust_int_sub", "op");
        let eq_op = field_ref(&boot, "rust_int_eq", "op");
        let table = RealizationCostTable::for_language(&boot, rust_language)
            .expect("rust realization-cost table should build from LanguageSpec rows");

        let user = compile_to_dag(
            "\
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)

let demo: Int = countdown(3) + 1
",
            "r3_gate70_cost_lens.v3",
        )
        .expect("representative recursive program compiles");
        let emitted = emit_rust(&user).expect("representative program emits to Rust");
        assert!(
            emitted.contains("fn countdown")
                && emitted.contains("countdown(&(((*(p0)) - 1)))")
                && emitted.contains("let demo: i64 = (countdown(&(3)) + 1);"),
            "emitted Rust should realize the recursive countdown target program:\n{emitted}"
        );

        let mut saw_add = false;
        let mut saw_sub = false;
        let mut saw_eq = false;
        for transform in user.nodes().iter().filter_map(Behavior::as_transform) {
            match transform.target {
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)) => {
                    saw_add = true;
                }
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Sub)) => {
                    saw_sub = true;
                }
                TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)) => {
                    saw_eq = true;
                }
                _ => {}
            }
        }
        assert!(
            saw_add && saw_sub && saw_eq,
            "gate #70 fixture should expose Add/Sub/Eq algebra instances; got add={saw_add}, sub={saw_sub}, eq={saw_eq}"
        );

        let descent_entries = per_call_descent_evidence(&user);
        assert!(
            descent_entries
                .iter()
                .any(|entry| entry.caller == "countdown" && entry.callee == "countdown"),
            "gate #70 fixture should expose a recursive countdown call, got {descent_entries:?}"
        );

        let countdown = find_bind_value(&user, "countdown");
        let algebra_cost = match symbolic_cost_of(&user, &countdown) {
            SymbolicCostLookup::Hit(c) => c,
            SymbolicCostLookup::Miss => panic!("symbolic_cost_of Miss for `countdown`"),
        };
        assert!(
            mentions_linear(&algebra_cost),
            "recursive countdown should expose an observable linear cost bound, got {algebra_cost:?}"
        );

        let type_cost = table
            .cost(&RealizationCostKey::Type(int_decl))
            .expect("Rust Int realization cost")
            .value();
        let add_cost = table
            .cost(&RealizationCostKey::Operator {
                target: int_decl,
                op: add_op,
            })
            .expect("Rust Int add realization cost")
            .value();
        let sub_cost = table
            .cost(&RealizationCostKey::Operator {
                target: int_decl,
                op: sub_op,
            })
            .expect("Rust Int sub realization cost")
            .value();
        let eq_cost = table
            .cost(&RealizationCostKey::Operator {
                target: int_decl,
                op: eq_op,
            })
            .expect("Rust Int eq realization cost")
            .value();
        assert_eq!(
            (type_cost, add_cost, sub_cost, eq_cost),
            (1, 1, 1, 1),
            "fixture rows should expose Rust Int/Add/Sub/Eq realization costs"
        );

        let composed = [type_cost, add_cost, sub_cost, eq_cost]
            .into_iter()
            .fold(algebra_cost, |acc, cost| {
                sequential(acc, SymbolicCost::ConstantCost { _0: cost })
            });
        assert!(
            mentions_linear(&composed),
            "cost lens demo should preserve the observable linear bound while folding Rust realization rows, got {composed:?}"
        );
    });
}

/// Gate #37 — `CallableRealization` row: lowered `cost` field is readable (bootstrap `rust_is_empty_callable`).
/// Does **not** assert `symbolic_cost_of` × `sequential` on this row; see program plan gate **#37** wording.
#[test]
fn callable_realization_row_cost_readable_on_bootstrap() {
    let boot = bootstrap_dag();
    let cr_meta = callable_realization_meta(&boot).expect("CallableRealization meta");
    let row = boot
        .declaration_by_name("rust_is_empty_callable")
        .expect("rust_is_empty_callable row");
    assert_eq!(row.meta_tag, Some(cr_meta.id));
    assert_eq!(realization_row_cost_int(row), 1);
}

fn named_id(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("missing declaration `{name}`"))
        .id
}

fn bootstrap_dag() -> Dag {
    std::thread::Builder::new()
        .name("cost-target-realization-bootstrap".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(generated_full_bootstrap_dag)
        .expect("spawn bootstrap builder")
        .join()
        .expect("bootstrap builder should not panic")
}

fn field_ref(dag: &Dag, decl_name: &str, field_name: &str) -> DeclarationId {
    match field_value(dag, decl_name, field_name) {
        FieldValue::Reference(id) => *id,
        other => panic!("{decl_name}.{field_name} should be a DeclarationRef, got {other:?}"),
    }
}

fn field_value<'a>(dag: &'a Dag, decl_name: &str, field_name: &str) -> &'a FieldValue {
    let decl = dag
        .declaration_by_name(decl_name)
        .unwrap_or_else(|| panic!("missing declaration `{decl_name}`"));
    let Some(ValueBody::Structural { fields }) = &decl.value_body else {
        panic!("declaration `{decl_name}` should have structural value_body");
    };
    fields
        .iter()
        .find_map(|(label, value)| (label == field_name).then_some(value))
        .unwrap_or_else(|| panic!("missing field `{field_name}` on `{decl_name}`"))
}
