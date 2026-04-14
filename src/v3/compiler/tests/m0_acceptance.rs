// M0 acceptance tests.
//
// Tests 1-5 cover the full M0 pipeline. Structural audits verify
// the fail-closed invariant (C-8) holds across both happy-path and
// error-path inputs.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, FunctionRef, LiteralValue, PortState};
use v3_compiler::lens_provenance::{Origin, ProvenanceLens};
use v3_compiler::types::{Prim, TypeShape};
use v3_compiler::CompileError;

/// Test helper: extract the Dag regardless of whether the compile
/// succeeded or failed with semantic errors. Panics on tokenize/parse
/// failures (which would indicate an unexpected structural error
/// in the test source).
fn compile_any(src: &str, file: &str) -> Dag {
    match compile_to_dag(src, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}

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
fn test_provenance_lens_reads_produced_by() {
    let src = "let a = 1\nlet b = 2\nlet c = a + b";
    let dag = compile_to_dag(src, "test.v3").expect("compiles");
    let lens = ProvenanceLens::new(&dag);

    let bind_c = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "c")
        .expect("Bind(c) must exist");

    let add_id = match lens.origin_of(bind_c.value) {
        Origin::Computed { by } => by,
        other => panic!("expected Origin::Computed for c, got {other:?}"),
    };

    let add = dag
        .node(add_id)
        .as_transform()
        .expect("add_id points to a Transform");
    assert_eq!(add.target, FunctionRef::new("std::int::add"));

    for input in &add.inputs {
        match lens.origin_of(*input) {
            Origin::Source { by: Some(node_id) } => {
                let value_node = dag
                    .node(node_id)
                    .as_value()
                    .expect("the lens reported a Value source");
                assert!(matches!(value_node.data, LiteralValue::Int(_)));
            }
            other => panic!("expected Origin::Source from Value, got {other:?}"),
        }
    }
}

#[test]
fn test_type_mismatch_produces_diagnostic_entry() {
    // `let x: Bool = 1` — G5: compile_to_dag returns
    // Err(CompileError::Semantic(dag)) because the diagnostic table
    // is non-empty. The Dag is still accessible via the Err payload.
    // The port for the Value(1) is Unresolved; the diagnostic table
    // has a TypeMismatch entry.
    let result = compile_to_dag("let x: Bool = 1", "test.v3");
    let dag = match result {
        Err(CompileError::Semantic(dag)) => dag,
        other => panic!("expected Err(Semantic), got {other:?}"),
    };

    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) still exists");

    let port = dag.port(bind_x.value);
    assert!(
        matches!(port.state(), PortState::Unresolved),
        "type-mismatch port is Unresolved, state = {:?}",
        port.state()
    );
    assert!(
        dag.diagnostics().contains(port.id()),
        "the diagnostic table has an entry for the mismatched port"
    );
    let diag = dag.diagnostics().get(port.id()).unwrap();
    match diag {
        v3_compiler::Diagnostic::TypeMismatch {
            expected, actual, ..
        } => {
            assert_eq!(*expected, TypeShape::Primitive(Prim::Bool));
            assert_eq!(*actual, TypeShape::Primitive(Prim::Int));
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn test_forward_reference_produces_diagnostic() {
    // Fail-closed (C-8): forward references are a failure path
    // routed through mark_unresolved, not a panic. The placeholder
    // port is Unresolved; compile_to_dag returns Err(Semantic).
    let dag = compile_any("let y = x\nlet x = 1", "test.v3");
    let bind_y = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "y")
        .expect("Bind(y) still exists");
    assert!(
        matches!(dag.port(bind_y.value).state(), PortState::Unresolved),
        "forward-ref port is Unresolved"
    );
    assert!(
        dag.diagnostics().contains(bind_y.value),
        "forward-ref has a diagnostic entry"
    );
}

#[test]
fn test_arity_mismatch_produces_diagnostic() {
    // Fail-closed (C-8): decide-level failure in infer routes
    // through mark_unresolved, not a silent return.
    let dag = compile_any(
        "fn f(a: Int) -> Int = a\nlet x = f(1, 2)",
        "test.v3",
    );
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) still exists");
    assert!(
        matches!(dag.port(bind_x.value).state(), PortState::Unresolved),
        "arity-mismatched call is Unresolved"
    );
    let diag = dag
        .diagnostics()
        .get(bind_x.value)
        .expect("diagnostic recorded");
    assert!(
        matches!(diag, v3_compiler::Diagnostic::ArityMismatch { .. }),
        "arity mismatch diagnostic, got {diag:?}"
    );
}

#[test]
fn test_unknown_function_produces_diagnostic() {
    let dag = compile_any("let x = unknown_fn(1)", "test.v3");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) still exists");
    assert!(
        matches!(dag.port(bind_x.value).state(), PortState::Unresolved),
        "unknown function call is Unresolved"
    );
    let diag = dag
        .diagnostics()
        .get(bind_x.value)
        .expect("diagnostic recorded");
    assert!(
        matches!(diag, v3_compiler::Diagnostic::ResolveError { .. }),
        "resolve error, got {diag:?}"
    );
}

#[test]
fn test_compile_boundary_is_fail_closed() {
    // compile_to_dag returns Ok ONLY when the diagnostic table is
    // empty. A happy-path source returns Ok; any error source must
    // return Err, even when the Dag is still well-formed enough to
    // inspect.
    assert!(
        compile_to_dag("let x = 1 + 2", "test.v3").is_ok(),
        "clean compile returns Ok"
    );
    assert!(
        matches!(
            compile_to_dag("let x: Bool = 1", "test.v3"),
            Err(CompileError::Semantic(_))
        ),
        "type-mismatch source returns Err(Semantic)"
    );
    assert!(
        matches!(
            compile_to_dag("let y = x\nlet x = 1", "test.v3"),
            Err(CompileError::Semantic(_))
        ),
        "forward-reference source returns Err(Semantic)"
    );
}

#[test]
fn invariant_port_state_matches_diagnostic_table() {
    // Structural audit of the enforced mark_unresolved API.
    //
    //   PortState == Unresolved       iff  DiagnosticTable.contains(port)
    //   PortState == Resolved(_)      implies !DiagnosticTable.contains(port)
    //   PortState == Uninferred       MUST NOT exist after compile returns
    //
    // Runs over both happy-path AND error-path inputs so the
    // biconditional is verified under conditions that actually
    // exercise each state.
    let sources = &[
        // Happy path
        "let x = 1 + 2",
        "let x = 5\nlet result = if x > 0 then 1 else 2",
        "fn count_down(n: Int) -> Int = if n == 0 then 0 else n + count_down(n - 1)\nlet answer = count_down(3)",
        "let a = 1\nlet b = 2\nlet c = a + b",
        // Error paths
        "let x: Bool = 1",
        "let y = x\nlet x = 1",
        "fn f(a: Int) -> Int = a\nlet x = f(1, 2)",
        "let x = unknown_fn(1)",
    ];
    for src in sources {
        let dag = compile_any(src, "invariant.v3");
        for port in dag.all_ports() {
            match port.state() {
                PortState::Uninferred => {
                    panic!(
                        "port {:?} is Uninferred after compile — post-sweep failed \
                         in source {src:?}",
                        port.id()
                    );
                }
                PortState::Resolved(_) => {
                    assert!(
                        !dag.diagnostics().contains(port.id()),
                        "Resolved port {:?} has a diagnostic entry in source {src:?}",
                        port.id(),
                    );
                }
                PortState::Unresolved => {
                    assert!(
                        dag.diagnostics().contains(port.id()),
                        "Unresolved port {:?} has no diagnostic entry in source {src:?}",
                        port.id(),
                    );
                }
            }
        }
    }
}
