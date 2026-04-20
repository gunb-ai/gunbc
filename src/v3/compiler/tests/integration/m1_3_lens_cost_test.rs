// M1(3) PR-B — cost lens integration receipts.
//
// Structural direct-Dag cases live in in-crate unit tests so they can
// use the crate-private builder surface without widening the public API.
// This file keeps only compile-path receipts that still need real
// lowering or stdlib callable wiring. Direct-Dag structural claims
// such as branch max-vs-sum behavior live in `src/lib.rs::lens_cost::tests`.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, PortId};
use v3_compiler::lens_cost::cost_of;

fn find_bind_value(dag: &v3_compiler::dag::Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn expect_cost(dag: &v3_compiler::dag::Dag, port: PortId) -> usize {
    crate::common::require_fixture_cost_usize(cost_of(dag, &port), &format!("port {port:?}"))
}

fn bind_cost(dag: &v3_compiler::dag::Dag, name: &str) -> usize {
    expect_cost(dag, find_bind_value(dag, name))
}

#[test]
fn kf_1_recursive_function_has_nonzero_cost() {
    let literal = compile_to_dag("fn constant(n: Int) -> Int = 0", "kf_1_constant.v3")
        .expect("literal-bodied function compiles");
    let recursive = compile_to_dag(
        "\
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)
",
        "kf_1_recursive.v3",
    )
    .expect("recursive function compiles");

    let literal_cost = bind_cost(&literal, "constant");
    let recursive_cost = bind_cost(&recursive, "countdown");

    assert!(
        recursive_cost > literal_cost,
        "recursive function should cost more structurally than a literal body: literal={literal_cost}, recursive={recursive_cost}"
    );
}

#[test]
fn kf_1_nested_fold_costs_more_than_flat_fold() {
    let flat = compile_to_dag(
        "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
        "kf_1_flat_fold.v3",
    )
    .expect("flat fold compiles");
    let nested = compile_to_dag(
        "let total: Int = fold(map(singleton(1), |x| x + 1), 0, |acc, x| acc + x)",
        "kf_1_nested_fold.v3",
    )
    .expect("nested fold compiles");

    let flat_cost = bind_cost(&flat, "total");
    let nested_cost = bind_cost(&nested, "total");

    assert!(
        nested_cost > flat_cost,
        "nested fold should cost more structurally than flat fold: flat={flat_cost}, nested={nested_cost}"
    );
}

#[test]
#[ignore = "blocked on lambda-aware higher-order cost attribution in CostLens"]
fn kf_1_lambda_body_cost_contributes_to_fold() {
    let simple = compile_to_dag(
        "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
        "kf_1_fold_lambda_simple.v3",
    )
    .expect("simple fold compiles");
    let richer = compile_to_dag(
        "let total: Int = fold(singleton(1), 0, |acc, x| acc + x + x)",
        "kf_1_fold_lambda_richer.v3",
    )
    .expect("richer fold compiles");

    let simple_cost = bind_cost(&simple, "total");
    let richer_cost = bind_cost(&richer, "total");

    assert!(
        richer_cost > simple_cost,
        "a larger lambda body should increase enclosing fold cost: simple={simple_cost}, richer={richer_cost}"
    );
}

#[test]
fn kf_1_list_operation_cost_ordering() {
    let singleton =
        compile_to_dag("let xs = singleton(1)", "kf_1_singleton.v3").expect("singleton compiles");
    let cons =
        compile_to_dag("let xs = cons(1, singleton(2))", "kf_1_cons.v3").expect("cons compiles");
    let fold = compile_to_dag(
        "let total: Int = fold(cons(1, singleton(2)), 0, |acc, x| acc + x)",
        "kf_1_fold.v3",
    )
    .expect("fold compiles");

    let singleton_cost = bind_cost(&singleton, "xs");
    let cons_cost = bind_cost(&cons, "xs");
    let fold_cost = bind_cost(&fold, "total");

    assert!(
        cons_cost > singleton_cost,
        "cons should cost more structurally than singleton: singleton={singleton_cost}, cons={cons_cost}"
    );
    assert!(
        fold_cost > cons_cost,
        "fold should cost more structurally than cons: cons={cons_cost}, fold={fold_cost}"
    );
}
