// M1(3) PR-B — cost lens acceptance tests.
//
// The cost lens is PR-B's third observational lens. These tests
// anchor its semantics against concrete programs: leaves cost 0,
// each Transform/Branch/Loop costs 1 plus its recursive cost, and
// Branch uses max over paths (not sum) because runtime fires
// exactly one path.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Behavior;
use v3_compiler::lens_cost::CostLens;

fn find_bind_value(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

#[test]
fn cost_lens_literal_value_is_zero() {
    // `let x = 1`
    //   Value(1) is a leaf — zero work.
    let dag = compile_to_dag("let x = 1", "test.v3").expect("compiles");
    let lens = CostLens::new(&dag);
    assert_eq!(lens.cost_of(find_bind_value(&dag, "x")), 0);
}

#[test]
fn cost_lens_single_transform_is_one() {
    // `let x = 1 + 2`
    //   Value(1) cost 0, Value(2) cost 0
    //   Add cost = 1 + (0 + 0) = 1
    let dag = compile_to_dag("let x = 1 + 2", "test.v3").expect("compiles");
    let lens = CostLens::new(&dag);
    assert_eq!(lens.cost_of(find_bind_value(&dag, "x")), 1);
}

#[test]
fn cost_lens_chained_transform_is_two() {
    // `let x = 1 + 2 + 3`
    //   left-associative: ((1 + 2) + 3)
    //   inner Add cost 1, Value(3) cost 0
    //   outer Add cost = 1 + (1 + 0) = 2
    let dag = compile_to_dag("let x = 1 + 2 + 3", "test.v3").expect("compiles");
    let lens = CostLens::new(&dag);
    assert_eq!(lens.cost_of(find_bind_value(&dag, "x")), 2);
}

#[test]
fn cost_lens_branch_counts_condition_plus_max_path() {
    // `let r = if 1 > 0 then 10 else 20`
    //   condition: Value(1), Value(0) -> Gt, cost 1
    //   then:      Value(10), cost 0
    //   else:      Value(20), cost 0
    //   Branch cost = 1 + cond + max(paths) = 1 + 1 + 0 = 2
    let dag = compile_to_dag(
        "let r = if 1 > 0 then 10 else 20",
        "test.v3",
    )
    .expect("compiles");
    let lens = CostLens::new(&dag);
    assert_eq!(lens.cost_of(find_bind_value(&dag, "r")), 2);
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
    let dag = compile_to_dag(
        "let r = if 1 > 0 then 20 + 30 else 40 + 50 + 60",
        "test.v3",
    )
    .expect("compiles");
    let lens = CostLens::new(&dag);
    // 1 (branch) + 1 (cond Gt) + max(1, 2) = 4
    assert_eq!(lens.cost_of(find_bind_value(&dag, "r")), 4);
}

#[test]
fn cost_lens_bind_passes_through_to_value() {
    // `let y = 1 + 2`
    //   cost_of(bind_y.value) reads the Add's output port directly
    //   cost_of takes the PortId, so we never see a Bind produce it
    //   (Bind's value field IS the underlying port). A passthrough
    //   test verifies this: bind_y cost equals direct Add cost.
    let dag = compile_to_dag("let y = 1 + 2", "test.v3").expect("compiles");
    let lens = CostLens::new(&dag);
    assert_eq!(lens.cost_of(find_bind_value(&dag, "y")), 1);
}
