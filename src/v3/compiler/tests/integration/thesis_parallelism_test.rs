use std::collections::HashSet;

use crate::common::cached_compile_to_dag;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, PortId, TransformTarget};
use v3_compiler::operators::{ArithmeticOp, OperatorKind};

fn bind_named<'a>(dag: &'a Dag, name: &str) -> &'a v3_compiler::dag::BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
}

fn anonymous_lambda_binds(dag: &Dag) -> Vec<&v3_compiler::dag::BindNode> {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .filter(|bind| bind.name.starts_with("__anon_lambda_"))
        .collect()
}

fn behavior_inputs(dag: &Dag, behavior: &Behavior) -> Vec<PortId> {
    match behavior {
        Behavior::Value(_) => Vec::new(),
        Behavior::Transform(t) => t.inputs.clone(),
        Behavior::Branch(b) => {
            let mut inputs = vec![b.input];
            inputs.extend(b.paths.iter().map(|path| path.output));
            inputs
        }
        Behavior::Loop(l) => {
            let mut inputs = vec![l.source, l.init];
            if let Some(count) = l.bound.count_port() {
                inputs.push(count);
            }
            inputs.push(match dag.node(l.body) {
                Behavior::Value(v) => v.output,
                Behavior::Transform(t) => t.output,
                Behavior::Branch(b) => b.output,
                Behavior::Loop(inner) => inner.output,
                Behavior::Bind(b) => b.value,
            });
            inputs
        }
        Behavior::Bind(b) => vec![b.value],
    }
}

fn has_transitive_dependency(dag: &Dag, from_port: PortId, to_port: PortId) -> bool {
    let mut seen: HashSet<PortId> = HashSet::new();
    let mut queue = vec![from_port];
    while let Some(port) = queue.pop() {
        if !seen.insert(port) {
            continue;
        }
        if port == to_port {
            return true;
        }
        let Some(producer) = dag.port(port).produced_by else {
            continue;
        };
        queue.extend(behavior_inputs(dag, dag.node(producer)));
    }
    false
}

#[test]
fn parallel_independent_bindings_have_no_dependency() {
    let dag = cached_compile_to_dag(
        "\
let a: Int = 1 + 2
let b: Int = 3 + 4
",
        "parallel_independent_bindings.v3",
    );
    let a = bind_named(&dag, "a");
    let b = bind_named(&dag, "b");

    assert!(
        !has_transitive_dependency(&dag, a.value, b.value),
        "`a` should not depend on `b`"
    );
    assert!(
        !has_transitive_dependency(&dag, b.value, a.value),
        "`b` should not depend on `a`"
    );
}

#[test]
fn sequential_dependent_bindings_have_dependency() {
    let dag = cached_compile_to_dag(
        "\
let a: Int = 1 + 2
let b: Int = a + 3
",
        "parallel_dependent_bindings.v3",
    );
    let a = bind_named(&dag, "a");
    let b = bind_named(&dag, "b");

    assert!(
        has_transitive_dependency(&dag, b.value, a.value),
        "`b` should depend transitively on `a`"
    );
    assert!(
        !has_transitive_dependency(&dag, a.value, b.value),
        "`a` should not depend on `b`"
    );
}

#[test]
fn transitive_dependencies_are_detected_across_multiple_steps() {
    let dag = cached_compile_to_dag(
        "\
let a: Int = 1
let b: Int = a + 1
let c: Int = b + 1
",
        "parallel_transitive_bindings.v3",
    );
    let a = bind_named(&dag, "a");
    let b = bind_named(&dag, "b");
    let c = bind_named(&dag, "c");

    assert!(
        has_transitive_dependency(&dag, c.value, b.value),
        "`c` should depend directly on `b`"
    );
    assert!(
        has_transitive_dependency(&dag, c.value, a.value),
        "`c` should depend transitively on `a`"
    );
    assert!(
        !has_transitive_dependency(&dag, a.value, c.value),
        "`a` should not depend on `c`"
    );
}

#[test]
fn diamond_dependencies_preserve_shared_parent_without_serializing_siblings() {
    let dag = cached_compile_to_dag(
        "\
let a: Int = 1
let b: Int = a + 1
let c: Int = a + 2
let d: Int = b + c
",
        "parallel_diamond_bindings.v3",
    );
    let a = bind_named(&dag, "a");
    let b = bind_named(&dag, "b");
    let c = bind_named(&dag, "c");
    let d = bind_named(&dag, "d");

    assert!(
        !has_transitive_dependency(&dag, b.value, c.value),
        "`b` should not depend on sibling branch `c`"
    );
    assert!(
        !has_transitive_dependency(&dag, c.value, b.value),
        "`c` should not depend on sibling branch `b`"
    );
    assert!(
        has_transitive_dependency(&dag, b.value, a.value),
        "`b` should depend on shared parent `a`"
    );
    assert!(
        has_transitive_dependency(&dag, c.value, a.value),
        "`c` should depend on shared parent `a`"
    );
    assert!(
        has_transitive_dependency(&dag, d.value, b.value)
            && has_transitive_dependency(&dag, d.value, c.value),
        "`d` should depend on both diamond branches"
    );
}

#[test]
fn parallel_map_elements_are_independent() {
    let dag = compile_to_dag(
        "let ys = map(singleton(1), |x| x + 1)",
        "parallel_map_elements.v3",
    )
    .expect("compiles");
    let lambda = anonymous_lambda_binds(&dag)
        .into_iter()
        .next()
        .expect("map should lower one anonymous lambda bind");
    let ys = bind_named(&dag, "ys");
    let element_param = *lambda
        .params
        .last()
        .expect("map lambda should expose one element parameter");

    assert_eq!(
        lambda.params.len(),
        1,
        "map lambda should not carry an accumulator or prior-iteration input"
    );
    assert!(
        has_transitive_dependency(&dag, lambda.value, element_param),
        "map lambda body should depend on its current element"
    );
    assert!(
        !has_transitive_dependency(&dag, ys.value, lambda.value),
        "mapped result should not feed back into the per-element lambda body"
    );
}

#[test]
fn sequential_fold_accumulator_chains_iterations() {
    let dag = compile_to_dag(
        "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
        "sequential_fold_accumulator.v3",
    )
    .expect("compiles");
    let lambda = anonymous_lambda_binds(&dag)
        .into_iter()
        .next()
        .expect("fold should lower one anonymous lambda bind");
    let acc_param = lambda.params[lambda.params.len() - 2];
    let elem_param = lambda.params[lambda.params.len() - 1];

    assert!(
        lambda.params.len() >= 2,
        "fold lambda should expose accumulator and element parameters"
    );
    assert!(
        has_transitive_dependency(&dag, lambda.value, acc_param),
        "fold lambda body should depend on the previous accumulator value"
    );
    assert!(
        has_transitive_dependency(&dag, lambda.value, elem_param),
        "fold lambda body should also depend on the current element"
    );
}

#[test]
fn fold_body_can_contain_accumulator_independent_subgraphs() {
    let dag = compile_to_dag(
        "let total: Int = fold(singleton(1), 0, |acc, x| acc + x * x)",
        "fold_acc_independent_subgraph.v3",
    )
    .expect("compiles");
    let lambda = anonymous_lambda_binds(&dag)
        .into_iter()
        .next()
        .expect("fold should lower one anonymous lambda bind");
    let acc_param = lambda.params[lambda.params.len() - 2];
    let elem_param = lambda.params[lambda.params.len() - 1];
    let mul_output = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_transform)
        .find(|transform| {
            matches!(
                transform.target,
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Mul))
            ) && has_transitive_dependency(&dag, lambda.value, transform.output)
        })
        .map(|transform| transform.output)
        .expect("expected a multiplication subgraph inside the fold body");

    assert!(
        has_transitive_dependency(&dag, mul_output, elem_param),
        "the x * x subgraph should depend on the current element"
    );
    assert!(
        !has_transitive_dependency(&dag, mul_output, acc_param),
        "the x * x subgraph should not depend on the accumulator"
    );
}

#[test]
fn independent_function_calls_stay_independent_across_bindings() {
    let dag = cached_compile_to_dag(
        "\
fn f(x: Int) -> Int = x + 1
fn g(x: Int) -> Int = x + 2
let a: Int = f(1)
let b: Int = g(2)
",
        "parallel_cross_function_bindings.v3",
    );
    let a = bind_named(&dag, "a");
    let b = bind_named(&dag, "b");

    assert!(
        !has_transitive_dependency(&dag, a.value, b.value),
        "`a` should stay independent from `b` across function boundaries"
    );
    assert!(
        !has_transitive_dependency(&dag, b.value, a.value),
        "`b` should stay independent from `a` across function boundaries"
    );
}

#[test]
#[ignore = "blocked on L2 M1 algebra awareness for commutative-monoid reduction"]
fn parallel_fold_on_commutative_monoid_is_reducible() {
    let dag = compile_to_dag(
        "let total: Int = fold(cons(1, cons(2, singleton(3))), 0, |acc, x| acc + x)",
        "parallel_fold_commutative_monoid.v3",
    )
    .expect("compiles");
    let _ = dag;
    panic!("blocked on algebra-aware reduction planning");
}
