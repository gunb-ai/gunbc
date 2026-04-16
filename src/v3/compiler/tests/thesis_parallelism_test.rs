use std::collections::HashSet;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, PortId};

fn bind_named<'a>(dag: &'a Dag, name: &str) -> &'a v3_compiler::dag::BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
}

fn anonymous_lambda_binds<'a>(dag: &'a Dag) -> Vec<&'a v3_compiler::dag::BindNode> {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .filter(|bind| bind.name.starts_with("__anon_lambda_"))
        .collect()
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
        match dag.node(producer) {
            Behavior::Value(_) => {}
            Behavior::Transform(t) => queue.extend(t.inputs.iter().copied()),
            Behavior::Branch(b) => {
                queue.push(b.input);
                queue.extend(b.paths.iter().map(|path| path.output));
            }
            Behavior::Loop(l) => {
                queue.push(l.source);
                queue.push(l.init);
                queue.push(l.bound.count);
                queue.push(match dag.node(l.body) {
                    Behavior::Value(v) => v.output,
                    Behavior::Transform(t) => t.output,
                    Behavior::Branch(b) => b.output,
                    Behavior::Loop(inner) => inner.output,
                    Behavior::Bind(b) => b.value,
                });
            }
            Behavior::Bind(b) => queue.push(b.value),
        }
    }
    false
}

#[test]
fn parallel_independent_bindings_have_no_dependency() {
    let dag = compile_to_dag(
        "\
let a: Int = 1 + 2
let b: Int = 3 + 4
",
        "parallel_independent_bindings.v3",
    )
    .expect("compiles");
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
    let dag = compile_to_dag(
        "\
let a: Int = 1 + 2
let b: Int = a + 3
",
        "parallel_dependent_bindings.v3",
    )
    .expect("compiles");
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
        !has_transitive_dependency(&dag, lambda.value, ys.value),
        "map lambda body should not depend on the whole mapped result structure"
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
