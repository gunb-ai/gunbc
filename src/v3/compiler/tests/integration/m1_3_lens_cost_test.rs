// M1(3) PR-B — cost lens acceptance tests.
//
// These tests now build the minimal graph shape the cost lens
// needs for its structural claims. Source compilation stays only
// where the receipt depends on real lowering or stdlib callable
// wiring rather than the lens's graph walk.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    Behavior, BranchPattern, Dag, LiteralBits, Path, PortId, TransformTarget,
};
use v3_compiler::diagnostics::SourceSpan;
use v3_compiler::lens_cost::cost_of;
use v3_compiler::operators::{ArithmeticOp, ComparisonOp, OperatorKind};

const DIRECT_DAG_FILE: &str = "m1_3_lens_cost_test.direct";

fn span() -> SourceSpan {
    SourceSpan::new(DIRECT_DAG_FILE, 0, 0)
}

fn find_bind_value(dag: &Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn expect_cost(dag: &Dag, port: PortId) -> usize {
    crate::common::require_fixture_cost_usize(cost_of(dag, &port), &format!("port {port:?}"))
}

fn bind_cost(dag: &Dag, name: &str) -> usize {
    expect_cost(dag, find_bind_value(dag, name))
}

fn int_value(dag: &mut Dag, value: i64) -> PortId {
    dag.push_value(LiteralBits::Int(value), span())
}

fn bool_value(dag: &mut Dag, value: bool) -> PortId {
    dag.push_value(LiteralBits::Bool(value), span())
}

fn add(dag: &mut Dag, lhs: PortId, rhs: PortId) -> PortId {
    dag.push_transform(
        TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
        vec![lhs, rhs],
        span(),
    )
}

fn gt(dag: &mut Dag, lhs: PortId, rhs: PortId) -> PortId {
    dag.push_transform(
        TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Gt)),
        vec![lhs, rhs],
        span(),
    )
}

fn bind_arm(dag: &mut Dag, name: &str, output: PortId) -> Path {
    let body = dag
        .port(output)
        .produced_by
        .unwrap_or_else(|| dag.push_bind(name, output, Vec::new(), span()));
    Path {
        body,
        output,
        pattern: BranchPattern::UnresolvedVariant {
            name: name.to_string(),
            span: span(),
        },
        binding: None,
    }
}

fn add_chain(dag: &mut Dag, values: &[i64]) -> PortId {
    let mut ports = values.iter().copied().map(|value| int_value(dag, value));
    let first = ports.next().expect("add_chain requires at least one literal");
    ports.fold(first, |lhs, rhs| add(dag, lhs, rhs))
}

fn branch_cost_fixture(then_values: &[i64], else_values: &[i64]) -> Dag {
    let mut dag = Dag::new();
    let cond = gt(&mut dag, int_value(&mut dag, 1), int_value(&mut dag, 0));
    let then_output = add_chain(&mut dag, then_values);
    let else_output = add_chain(&mut dag, else_values);
    let result = dag.push_branch(
        cond,
        vec![
            bind_arm(&mut dag, "then_arm", then_output),
            bind_arm(&mut dag, "else_arm", else_output),
        ],
        span(),
    );
    dag.push_bind("r", result, Vec::new(), span());
    dag
}

#[test]
fn cost_lens_literal_value_is_zero() {
    let mut dag = Dag::new();
    let value = int_value(&mut dag, 1);
    dag.push_bind("x", value, Vec::new(), span());

    assert_eq!(bind_cost(&dag, "x"), 0);
}

#[test]
fn cost_lens_single_transform_is_one() {
    let mut dag = Dag::new();
    let value = add(&mut dag, int_value(&mut dag, 1), int_value(&mut dag, 2));
    dag.push_bind("x", value, Vec::new(), span());

    assert_eq!(bind_cost(&dag, "x"), 1);
}

#[test]
fn cost_lens_chained_transform_is_two() {
    let mut dag = Dag::new();
    let inner = add(&mut dag, int_value(&mut dag, 1), int_value(&mut dag, 2));
    let value = add(&mut dag, inner, int_value(&mut dag, 3));
    dag.push_bind("x", value, Vec::new(), span());

    assert_eq!(bind_cost(&dag, "x"), 2);
}

#[test]
fn cost_lens_branch_counts_condition_plus_max_path() {
    let mut dag = Dag::new();
    let cond = gt(&mut dag, int_value(&mut dag, 1), int_value(&mut dag, 0));
    let then_output = int_value(&mut dag, 10);
    let else_output = int_value(&mut dag, 20);
    let result = dag.push_branch(
        cond,
        vec![
            bind_arm(&mut dag, "then_arm", then_output),
            bind_arm(&mut dag, "else_arm", else_output),
        ],
        span(),
    );
    dag.push_bind("r", result, Vec::new(), span());

    assert_eq!(bind_cost(&dag, "r"), 2);
}

#[test]
fn cost_lens_branch_uses_max_not_sum_across_paths() {
    let dag = branch_cost_fixture(&[20, 30], &[40, 50, 60]);

    assert_eq!(bind_cost(&dag, "r"), 4);
}

#[test]
fn cost_lens_bind_passes_through_to_value() {
    let mut dag = Dag::new();
    let add_output = add(&mut dag, int_value(&mut dag, 1), int_value(&mut dag, 2));
    dag.push_bind("y", add_output, Vec::new(), span());

    assert_eq!(bind_cost(&dag, "y"), expect_cost(&dag, add_output));
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
    let baseline = branch_cost_fixture(&[10, 20, 30, 40], &[50, 60]);
    let larger_non_max = branch_cost_fixture(&[10, 20, 30, 40], &[50, 60, 70]);
    let larger_max = branch_cost_fixture(&[10, 20, 30, 40, 50], &[60, 70]);

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
fn kf_1_non_max_branch_does_not_inflate_cost() {
    let baseline = branch_cost_fixture(&[1, 2, 3, 4], &[5, 6]);
    let more_non_max_work = branch_cost_fixture(&[1, 2, 3, 4], &[5, 6, 7, 8]);
    let more_max_work = branch_cost_fixture(&[1, 2, 3, 4, 5], &[6, 7]);

    let baseline_cost = bind_cost(&baseline, "r");
    let more_non_max_work_cost = bind_cost(&more_non_max_work, "r");
    let more_max_work_cost = bind_cost(&more_max_work, "r");

    assert_eq!(
        baseline_cost, more_non_max_work_cost,
        "growing only the non-max branch should not change structural cost: baseline={baseline_cost}, more_non_max_work={more_non_max_work_cost}"
    );
    assert!(
        more_max_work_cost > baseline_cost,
        "growing the max branch should increase structural cost: baseline={baseline_cost}, more_max_work={more_max_work_cost}"
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
    let singleton = compile_to_dag("let xs = singleton(1)", "kf_1_singleton.v3")
        .expect("singleton compiles");
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
