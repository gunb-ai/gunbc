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

use v3_compiler::dag::{Dag, DeclarationId, FieldValue, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::lens_cost_target_realization::{
    behavior_realization_meta, callable_realization_meta, operator_realization_meta,
    pattern_realization_meta, type_instantiation_realization_meta, type_realization_meta,
};
use v3_compiler::realization_cost::{
    RealizationCostCategory, RealizationCostKey, RealizationCostTable,
};

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
