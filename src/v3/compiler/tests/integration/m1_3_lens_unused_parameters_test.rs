// M1(3) — lens_unused_parameters acceptance tests.
//
// The structural cases in this file build the smallest Dag shape
// needed for each lens claim. Source compilation remains only for
// the parser-gap receipt because that test is about source support,
// not the lens walk itself.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{BranchPattern, Dag, LiteralBits, LoopBound, Path, PortId, TransformTarget};
use v3_compiler::dag_test_support as dag_test;
use v3_compiler::diagnostics::SourceSpan;
use v3_compiler::lens_unused_parameters::{UnusedParametersConfig, UnusedParametersLens};
use v3_compiler::operators::{ArithmeticOp, ComparisonOp, OperatorKind};

const DIRECT_DAG_FILE: &str = "m1_3_lens_unused_parameters_test.direct";

fn span() -> SourceSpan {
    SourceSpan::new(DIRECT_DAG_FILE, 0, 0)
}

fn unused_parameter_indexes(dag: &Dag) -> Vec<usize> {
    let lens = UnusedParametersLens::new(dag);
    let mut indexes: Vec<_> = lens
        .query(&UnusedParametersConfig::default())
        .into_iter()
        .map(|violation| violation.parameter_index)
        .collect();
    indexes.sort_unstable();
    indexes
}

fn int_value(dag: &mut Dag, value: i64) -> PortId {
    dag_test::push_value(dag, LiteralBits::Int(value), span())
}

fn add(dag: &mut Dag, lhs: PortId, rhs: PortId) -> PortId {
    dag_test::push_transform(
        dag,
        TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
        vec![lhs, rhs],
        span(),
    )
}

fn gt(dag: &mut Dag, lhs: PortId, rhs: PortId) -> PortId {
    dag_test::push_transform(
        dag,
        TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Gt)),
        vec![lhs, rhs],
        span(),
    )
}

fn int_params(dag: &mut Dag, count: usize) -> Vec<PortId> {
    let int_shape = dag.int_shape().expect("bootstrap Dag has Int");
    (0..count)
        .map(|_| dag_test::alloc_port_with_shape(dag, int_shape))
        .collect()
}

fn producer_or_bind(dag: &mut Dag, name: &str, output: PortId) -> v3_compiler::dag::NodeId {
    dag.port(output)
        .produced_by
        .unwrap_or_else(|| dag_test::push_bind(dag, name, output, Vec::new(), span()))
}

fn bind_arm(dag: &mut Dag, name: &str, output: PortId) -> Path {
    Path {
        body: producer_or_bind(dag, name, output),
        output,
        pattern: BranchPattern::UnresolvedVariant {
            name: name.to_string(),
            span: span(),
        },
        binding: None,
    }
}

fn function_dag<F>(name: &str, param_count: usize, build_body: F) -> Dag
where
    F: FnOnce(&mut Dag, &[PortId]) -> PortId,
{
    let mut dag = Dag::new();
    let params = int_params(&mut dag, param_count);
    let value = build_body(&mut dag, &params);
    dag_test::push_bind(&mut dag, name, value, params, span());
    dag
}

#[test]
fn unused_params_empty_for_function_using_every_parameter() {
    let dag = function_dag("add", 2, |dag, params| add(dag, params[0], params[1]));

    assert_eq!(
        unused_parameter_indexes(&dag),
        Vec::<usize>::new(),
        "every parameter should be reachable from the function result"
    );
}

#[test]
fn unused_params_reports_single_unused_parameter() {
    let dag = function_dag("first", 2, |_dag, params| params[0]);

    assert_eq!(
        unused_parameter_indexes(&dag),
        vec![1],
        "expected only the second parameter to be unused"
    );
}

#[test]
fn unused_params_reports_all_parameters_for_constant_body() {
    let dag = function_dag("always_one", 3, |dag, _params| int_value(dag, 1));

    assert_eq!(
        unused_parameter_indexes(&dag),
        vec![0, 1, 2],
        "constant bodies should leave every parameter unused"
    );
}

#[test]
fn unused_params_skips_value_bindings() {
    let mut dag = Dag::new();
    let lhs = int_value(&mut dag, 1);
    let rhs = int_value(&mut dag, 2);
    let value = add(&mut dag, lhs, rhs);
    dag_test::push_bind(&mut dag, "x", value, Vec::new(), span());

    assert_eq!(
        unused_parameter_indexes(&dag),
        Vec::<usize>::new(),
        "value bindings should be skipped because they have no parameters"
    );
}

#[test]
fn unused_params_handles_branch_in_body() {
    let dag = function_dag("pick", 2, |dag, params| {
        let zero = int_value(dag, 0);
        let cond = gt(dag, params[0], zero);
        let then_path = bind_arm(dag, "then_arm", params[0]);
        let else_path = bind_arm(dag, "else_arm", params[1]);
        dag_test::push_branch(dag, cond, vec![then_path, else_path], span())
    });

    assert_eq!(
        unused_parameter_indexes(&dag),
        Vec::<usize>::new(),
        "branch traversal should find parameters used across the condition and both arms"
    );
}

#[test]
fn unused_params_bootstrap_baseline_is_empty() {
    let dag = Dag::new();

    assert_eq!(
        unused_parameter_indexes(&dag),
        Vec::<usize>::new(),
        "bootstrap-only Dag should report zero unused parameters at v3 M1(3) scope"
    );
}

#[test]
fn unused_params_canonical_target_blocked_on_parser_gaps() {
    let src = "\
fn content_upsert(content: String, path: String) -> { written: Bool } {
  let matches = content == \"\"
  { written: !matches }
}
";
    let result = compile_to_dag(src, "patterns.v3");

    assert!(
        result.is_err(),
        "v3 unexpectedly parsed content_upsert verbatim; flip this test to a positive assertion"
    );
}

#[test]
fn unused_params_catches_content_upsert_synthetic_equivalent() {
    let dag = function_dag("content_upsert", 2, |dag, params| {
        let zero = int_value(dag, 0);
        add(dag, params[0], zero)
    });

    assert_eq!(
        unused_parameter_indexes(&dag),
        vec![1],
        "the synthetic content_upsert shape should report only the ignored second parameter"
    );
}

#[test]
fn unused_params_descends_into_loop_body_for_recursive_calls() {
    let dag = function_dag("count_down", 2, |dag, params| {
        let body_output = add(dag, params[0], params[1]);
        let count = int_value(dag, 1);
        let body = producer_or_bind(dag, "loop_body", body_output);
        dag_test::push_loop(
            dag,
            params[0],
            params[1],
            body,
            LoopBound::Cardinality { count },
            span(),
        )
    });

    assert_eq!(
        unused_parameter_indexes(&dag),
        Vec::<usize>::new(),
        "loop traversal should include both the loop body and the loop inputs"
    );
}

#[test]
fn unused_params_loop_body_descent_finds_param_only_used_in_recursion() {
    let dag = function_dag("count_down", 2, |dag, params| {
        let body_output = add(dag, params[0], params[1]);
        let init = int_value(dag, 0);
        let count = int_value(dag, 1);
        let body = producer_or_bind(dag, "loop_body", body_output);
        dag_test::push_loop(
            dag,
            params[0],
            init,
            body,
            LoopBound::Cardinality { count },
            span(),
        )
    });

    assert_eq!(
        unused_parameter_indexes(&dag),
        Vec::<usize>::new(),
        "parameters referenced only through the loop body should still count as used"
    );
}

#[test]
fn unused_params_reports_unused_in_branch_body() {
    let dag = function_dag("always_a", 2, |dag, params| {
        let zero = int_value(dag, 0);
        let cond = gt(dag, params[0], zero);
        let then_path = bind_arm(dag, "then_arm", params[0]);
        let else_path = bind_arm(dag, "else_arm", params[0]);
        dag_test::push_branch(dag, cond, vec![then_path, else_path], span())
    });

    assert_eq!(
        unused_parameter_indexes(&dag),
        vec![1],
        "the branch body uses only the first parameter, so the second should be reported"
    );
}
