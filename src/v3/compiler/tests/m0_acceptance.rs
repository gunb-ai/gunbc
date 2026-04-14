// M0 acceptance tests.
//
// Test 1 — the session target: `let x = 1 + 2` compiles to a DAG
// with Value(1) + Value(2) + Transform(BinaryOp(Add)) + Bind(x).
//
// invariant_none_type_implies_diagnostic — the structural audit of
// the enforced mark_unresolved API. Walks every port in every
// M0-test DAG and asserts the biconditional:
//   Port.value_type == None  iff  DiagnosticTable contains PortId
//
// Tests 2-5 (Branch, Loop, provenance lens, type-mismatch diagnostic)
// are deferred to future sessions per the session plan.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, FunctionRef, LiteralValue};
use v3_compiler::types::{Prim, TypeShape};

#[test]
fn test_let_binding_produces_dag_shape() {
    let dag = compile_to_dag("let x = 1 + 2", "test.v3").expect("compiles");

    assert_eq!(
        dag.nodes().len(),
        4,
        "Value(1) + Value(2) + Transform(Add) + Bind(x)"
    );

    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) must exist");

    let value_port = dag.port(bind.value);
    let producer_id = value_port
        .produced_by
        .expect("Bind value has a producer node");
    let add = dag
        .node(producer_id)
        .as_transform()
        .expect("producer is a Transform");
    assert_eq!(add.target, FunctionRef::new("std::int::add"));
    assert_eq!(add.inputs.len(), 2);

    let lhs_producer = dag
        .port(add.inputs[0])
        .produced_by
        .expect("lhs port has a producer");
    let rhs_producer = dag
        .port(add.inputs[1])
        .produced_by
        .expect("rhs port has a producer");
    let lhs = dag.node(lhs_producer).as_value().expect("lhs is Value");
    let rhs = dag.node(rhs_producer).as_value().expect("rhs is Value");
    assert_eq!(lhs.data, LiteralValue::Int(1));
    assert_eq!(rhs.data, LiteralValue::Int(2));

    assert_eq!(
        dag.port(add.output).value_type(),
        Some(&TypeShape::Primitive(Prim::Int)),
        "inference propagated Int through the Add output port",
    );

    assert!(
        dag.diagnostics().is_empty(),
        "clean compile, no diagnostics"
    );
}

#[test]
fn test_if_then_else_produces_branch_dag() {
    let src = "let x = 5\nlet result = if x > 0 then 1 else 2";
    let dag = compile_to_dag(src, "test.v3").expect("compiles");

    let result_bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "result")
        .expect("Bind(result) must exist");

    let value_port = dag.port(result_bind.value);
    let branch_id = value_port
        .produced_by
        .expect("result has a producer node");
    let branch = dag
        .node(branch_id)
        .as_branch()
        .expect("producer is a Branch");

    assert_eq!(branch.paths.len(), 2, "if/else produces two paths");

    let cmp_id = dag
        .port(branch.input)
        .produced_by
        .expect("branch input has producer");
    let cmp = dag
        .node(cmp_id)
        .as_transform()
        .expect("branch input is a Transform");
    assert_eq!(cmp.target, FunctionRef::new("std::int::gt"));

    assert_eq!(
        dag.port(branch.input).value_type(),
        Some(&TypeShape::Primitive(Prim::Bool)),
        "comparison produces Bool",
    );

    assert_eq!(
        dag.port(branch.output).value_type(),
        Some(&TypeShape::Primitive(Prim::Int)),
        "both paths produce Int; unified branch output is Int",
    );

    for path in &branch.paths {
        assert_eq!(
            dag.port(path.output).value_type(),
            Some(&TypeShape::Primitive(Prim::Int)),
            "each path's output port is typed Int",
        );
    }

    assert!(dag.diagnostics().is_empty());
}

#[test]
fn test_recursive_function_produces_loop_dag() {
    let src = "fn count_down(n: Int) -> Int = if n == 0 then 0 else n + count_down(n - 1)\nlet answer = count_down(3)";
    let dag = compile_to_dag(src, "test.v3").expect("compiles");

    let count_down = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "count_down")
        .expect("Bind(count_down) must exist");
    assert_eq!(count_down.params.len(), 1, "count_down has one parameter");

    let param_port = count_down.params[0];
    assert_eq!(
        dag.port(param_port).value_type(),
        Some(&TypeShape::Primitive(Prim::Int)),
        "parameter n annotated as Int",
    );

    let value_port = dag.port(count_down.value);
    let loop_id = value_port
        .produced_by
        .expect("count_down value has a producing node");
    let loop_node = dag
        .node(loop_id)
        .as_loop()
        .expect("producer is a Loop (bounded recursion)");

    assert_eq!(
        loop_node.bound.count, param_port,
        "Loop.bound.count chains back to the parameter n",
    );

    let branch = dag
        .node(loop_node.body)
        .as_branch()
        .expect("Loop body is a Branch (the if expression)");
    assert_eq!(branch.paths.len(), 2, "if/else produces two paths");

    let answer = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "answer")
        .expect("Bind(answer) must exist");

    let call_id = dag
        .port(answer.value)
        .produced_by
        .expect("answer has a producing node");
    let call = dag
        .node(call_id)
        .as_transform()
        .expect("call site is a Transform");
    assert_eq!(call.target, FunctionRef::new("count_down"));
    assert_eq!(call.inputs.len(), 1);

    assert_eq!(
        dag.port(answer.value).value_type(),
        Some(&TypeShape::Primitive(Prim::Int)),
        "answer is typed Int via count_down's declared return type",
    );

    assert!(dag.diagnostics().is_empty());
}

#[test]
fn invariant_none_type_implies_diagnostic() {
    // Structural audit of the enforced mark_unresolved API.
    //   Port.value_type == None  iff  DiagnosticTable contains PortId
    let sources = &[
        "let x = 1 + 2",
        "let x = 5\nlet result = if x > 0 then 1 else 2",
        "fn count_down(n: Int) -> Int = if n == 0 then 0 else n + count_down(n - 1)\nlet answer = count_down(3)",
    ];
    for src in sources {
        let dag = compile_to_dag(src, "invariant.v3").unwrap();
        for port in dag.all_ports() {
            assert_eq!(
                port.value_type().is_none(),
                dag.diagnostics().contains(port.id()),
                "physics invariant violated for port {:?} in source {src:?}",
                port.id(),
            );
        }
    }
}
