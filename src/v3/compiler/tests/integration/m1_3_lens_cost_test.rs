// M1(3) PR-B — cost lens acceptance tests.
//
// The cost lens is PR-B's third observational lens. These tests
// anchor its semantics against concrete programs: leaves cost 0,
// each Transform/Branch/Loop costs 1 plus its recursive cost, and
// Branch uses max over paths (not sum) because runtime fires
// exactly one path.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, PortId};
use v3_compiler::lens_cost::cost_of;

use crate::common::cached_compile_to_dag;
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
fn cost_lens_literal_value_is_zero() {
    // `let x = 1`
    //   Value(1) is a leaf — zero work.
    let dag = cached_compile_to_dag("let x = 1", "test.v3");
    assert_eq!(expect_cost(&dag, find_bind_value(&dag, "x")), 0);
}

#[test]
fn cost_lens_single_transform_is_one() {
    // `let x = 1 + 2`
    //   Value(1) cost 0, Value(2) cost 0
    //   Add cost = 1 + (0 + 0) = 1
    let dag = cached_compile_to_dag("let x = 1 + 2", "test.v3");
    assert_eq!(expect_cost(&dag, find_bind_value(&dag, "x")), 1);
}

#[test]
fn cost_lens_chained_transform_is_two() {
    // `let x = 1 + 2 + 3`
    //   left-associative: ((1 + 2) + 3)
    //   inner Add cost 1, Value(3) cost 0
    //   outer Add cost = 1 + (1 + 0) = 2
    let dag = cached_compile_to_dag("let x = 1 + 2 + 3", "test.v3");
    assert_eq!(expect_cost(&dag, find_bind_value(&dag, "x")), 2);
}

#[test]
fn cost_lens_branch_counts_condition_plus_max_path() {
    // `let r = if 1 > 0 then 10 else 20`
    //   condition: Value(1), Value(0) -> Gt, cost 1
    //   then:      Value(10), cost 0
    //   else:      Value(20), cost 0
    //   Branch cost = 1 + cond + max(paths) = 1 + 1 + 0 = 2
    let dag = cached_compile_to_dag("let r = if 1 > 0 then 10 else 20", "test.v3");
    assert_eq!(expect_cost(&dag, find_bind_value(&dag, "r")), 2);
}

#[test]
fn cost_lens_branch_uses_max_not_sum_across_paths() {
    // `let r = if 1 > 0 then 10 else 20 + 30 + 40`
    //   condition cost 1 (one Gt)
    //   then path cost 0 (just a literal)
    //   else path cost 2 (two Adds)
    //   Branch cost = 1 + 1 + max(0, 2) = 4
    //
    // If the lens summed paths instead of maxing, it would be
    // 1 + 1 + (0 + 2) = 4 — so we also need an asymmetric case
    // where max and sum differ. Use `20 + 30` vs `40 + 50 + 60`
    // to force the distinction: then=1, else=2, max=2, sum=3.
    let dag = compile_to_dag("let r = if 1 > 0 then 20 + 30 else 40 + 50 + 60", "test.v3")
        .expect("compiles");
    // 1 (branch) + 1 (cond Gt) + max(1, 2) = 4
    assert_eq!(expect_cost(&dag, find_bind_value(&dag, "r")), 4);
}

#[test]
fn cost_lens_bind_passes_through_to_value() {
    // `let y = 1 + 2`
    //   cost_of(bind_y.value) reads the Add's output port directly
    //   cost_of takes the PortId, so we never see a Bind produce it
    //   (Bind's value field IS the underlying port). A passthrough
    //   test verifies this: bind_y cost equals direct Add cost.
    let dag = cached_compile_to_dag("let y = 1 + 2", "test.v3");
    assert_eq!(expect_cost(&dag, find_bind_value(&dag, "y")), 1);
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
fn kf_1_branch_cost_is_max_not_sum() {
    let baseline = compile_to_dag(
        "let r: Int = if 1 > 0 then 10 + 20 + 30 + 40 else 50 + 60",
        "kf_1_branch_max_baseline.v3",
    )
    .expect("baseline branch compiles");
    let larger_non_max = compile_to_dag(
        "let r: Int = if 1 > 0 then 10 + 20 + 30 + 40 else 50 + 60 + 70",
        "kf_1_branch_non_max.v3",
    )
    .expect("larger non-max branch compiles");
    let larger_max = compile_to_dag(
        "let r: Int = if 1 > 0 then 10 + 20 + 30 + 40 + 50 else 60 + 70",
        "kf_1_branch_max.v3",
    )
    .expect("larger max branch compiles");

    let baseline_cost = bind_cost(&baseline, "r");
    let larger_non_max_cost = bind_cost(&larger_non_max, "r");
    let larger_max_cost = bind_cost(&larger_max, "r");

    assert_eq!(
        baseline_cost, larger_non_max_cost,
        "growing only the non-max path should leave branch cost unchanged: baseline={baseline_cost}, larger_non_max={larger_non_max_cost}"
    );
    assert!(
        larger_max_cost > baseline_cost,
        "growing the max path should increase branch cost: baseline={baseline_cost}, larger_max={larger_max_cost}"
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
fn kf_1_unused_branch_does_not_inflate_cost() {
    let baseline = compile_to_dag(
        "let r: Int = if true then 1 + 2 + 3 + 4 else 5 + 6",
        "kf_1_unused_branch_baseline.v3",
    )
    .expect("baseline branch compiles");
    let more_dead_work = compile_to_dag(
        "let r: Int = if true then 1 + 2 + 3 + 4 else 5 + 6 + 7 + 8",
        "kf_1_unused_branch_dead_work.v3",
    )
    .expect("larger dead branch compiles");
    let more_live_work = compile_to_dag(
        "let r: Int = if true then 1 + 2 + 3 + 4 + 5 else 6 + 7",
        "kf_1_unused_branch_live_work.v3",
    )
    .expect("larger live branch compiles");

    let baseline_cost = bind_cost(&baseline, "r");
    let more_dead_work_cost = bind_cost(&more_dead_work, "r");
    let more_live_work_cost = bind_cost(&more_live_work, "r");

    assert_eq!(
        baseline_cost, more_dead_work_cost,
        "growing only the unused branch should not change structural cost: baseline={baseline_cost}, more_dead_work={more_dead_work_cost}"
    );
    assert!(
        more_live_work_cost > baseline_cost,
        "growing the taken/max branch should increase structural cost: baseline={baseline_cost}, more_live_work={more_live_work_cost}"
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
    let singleton = cached_compile_to_dag("let xs = singleton(1)", "kf_1_singleton.v3");
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
