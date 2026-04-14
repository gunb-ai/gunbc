// M0 acceptance tests.
//
// Tests 1-5 cover the full M0 pipeline. Structural audits verify
// the fail-closed invariant (C-8) holds across both happy-path and
// error-path inputs.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, DeclarationId, LiteralBits, PortState};
use v3_compiler::lens_depth::DepthLens;
use v3_compiler::lens_provenance::{Origin, ProvenanceLens};
use v3_compiler::types::{Prim, TypeShape};
use v3_compiler::CompileError;

/// Assert that a Transform.target DeclarationId points to a declaration
/// whose `name` field equals `expected`. Replaces the M0-era
/// `add.target == FunctionRef::new("std::int::add")` shape with a walk
/// through the declaration table, matching the §8.9 target-as-Declaration
/// model.
fn assert_target_name(dag: &Dag, target: DeclarationId, expected: &str) {
    let decl = dag.declaration(target);
    assert_eq!(
        decl.name.as_deref(),
        Some(expected),
        "Transform.target declaration name mismatch"
    );
}

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
    assert_target_name(&dag, add.target, "+");
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
    assert_eq!(lhs.data, LiteralBits::Int(1));
    assert_eq!(rhs.data, LiteralBits::Int(2));

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
    assert_target_name(&dag, cmp.target, ">");

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
    assert_target_name(&dag, call.target, "count_down");
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
    assert_target_name(&dag, add.target, "+");

    for input in &add.inputs {
        match lens.origin_of(*input) {
            Origin::Source { by: Some(node_id) } => {
                let value_node = dag
                    .node(node_id)
                    .as_value()
                    .expect("the lens reported a Value source");
                assert!(matches!(value_node.data, LiteralBits::Int(_)));
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
fn test_bool_literal_true() {
    // `let x = true` — the value port producer is a Value node
    // carrying LiteralBits::Bool(true), and inference resolves the
    // port to Prim::Bool.
    let dag = compile_to_dag("let x = true", "test.v3").expect("compiles");

    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) must exist");

    let producer = dag
        .port(bind_x.value)
        .produced_by
        .expect("bind value has producer");
    let value_node = dag
        .node(producer)
        .as_value()
        .expect("producer is a Value node");
    assert_eq!(value_node.data, LiteralBits::Bool(true));

    assert_eq!(
        dag.port(bind_x.value).value_type(),
        Some(&TypeShape::Primitive(Prim::Bool)),
        "bool literal infers to Bool",
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn test_bool_literal_false() {
    let dag = compile_to_dag("let x = false", "test.v3").expect("compiles");

    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) must exist");

    let producer = dag
        .port(bind_x.value)
        .produced_by
        .expect("bind value has producer");
    let value_node = dag
        .node(producer)
        .as_value()
        .expect("producer is a Value node");
    assert_eq!(value_node.data, LiteralBits::Bool(false));

    assert_eq!(
        dag.port(bind_x.value).value_type(),
        Some(&TypeShape::Primitive(Prim::Bool)),
        "bool literal infers to Bool",
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn test_string_literal() {
    // `let x = "hello"` — the value port producer is a Value node
    // carrying LiteralBits::String("hello"), and inference resolves
    // the port to Prim::String.
    let dag = compile_to_dag("let x = \"hello\"", "test.v3").expect("compiles");

    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) must exist");

    let producer = dag
        .port(bind_x.value)
        .produced_by
        .expect("bind value has producer");
    let value_node = dag
        .node(producer)
        .as_value()
        .expect("producer is a Value node");
    assert_eq!(value_node.data, LiteralBits::String("hello".to_string()));

    assert_eq!(
        dag.port(bind_x.value).value_type(),
        Some(&TypeShape::Primitive(Prim::String)),
        "string literal infers to String",
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn test_bool_literal_in_conditional() {
    // `let x = if true then 1 else 2` — integration test. The
    // Branch node's `input` port must trace back to a Value(Bool(true))
    // producer, proving tokenize -> parse -> lower -> infer flows
    // bool literals end-to-end into the conditional context.
    let dag = compile_to_dag("let x = if true then 1 else 2", "test.v3")
        .expect("compiles");

    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) must exist");

    let branch_id = dag
        .port(bind_x.value)
        .produced_by
        .expect("bind value has producer");
    let branch = dag
        .node(branch_id)
        .as_branch()
        .expect("producer is a Branch");

    let cond_producer = dag
        .port(branch.input)
        .produced_by
        .expect("branch input has producer");
    let cond_value = dag
        .node(cond_producer)
        .as_value()
        .expect("branch condition is a Value node");
    assert_eq!(cond_value.data, LiteralBits::Bool(true));

    assert_eq!(
        dag.port(branch.input).value_type(),
        Some(&TypeShape::Primitive(Prim::Bool)),
        "branch condition is typed Bool",
    );
    assert_eq!(
        dag.port(branch.output).value_type(),
        Some(&TypeShape::Primitive(Prim::Int)),
        "unified branch output is Int (both arms are Int)",
    );
    assert!(dag.diagnostics().is_empty());
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

// ════════════════════════════════════════════════════════════════
// M0.7 Lane A — span correctness, Branch/Loop lens origins, and
// composition stress tests.
// ════════════════════════════════════════════════════════════════

#[test]
fn test_type_mismatch_span_points_at_value_expression() {
    // `let x: Bool = 1` — the TypeMismatch diagnostic's span should
    // point at the `1` literal, not at a synthetic location. Byte
    // offsets:
    //   l  e  t     x  :     B  o  o  l     =     1
    //   0  1  2  3  4  5  6  7  8  9 10 11 12 13 14
    // The value expression `1` is at byte [14, 15).
    let src = "let x: Bool = 1";
    let dag = compile_any(src, "test.v3");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
    let diag = dag
        .diagnostics()
        .get(bind_x.value)
        .expect("mismatch diagnostic recorded");
    let span = diag.span();
    assert_eq!(span.file, "test.v3");
    assert_eq!(
        span.byte_start, 14,
        "span points at the `1` literal, not a synthetic location"
    );
    assert_eq!(span.byte_end, 15);
}

#[test]
fn test_forward_reference_span_points_at_reference() {
    // `let y = x\nlet x = 1` — the ResolveError span should point
    // at the `x` reference, not at the `let y` prefix.
    //   l  e  t     y  =     x
    //   0  1  2  3  4  5  6  7
    // The `x` reference is at byte [8, 9).
    let src = "let y = x\nlet x = 1";
    let dag = compile_any(src, "test.v3");
    let bind_y = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "y")
        .expect("Bind(y) exists");
    let diag = dag
        .diagnostics()
        .get(bind_y.value)
        .expect("resolve error recorded");
    let span = diag.span();
    assert_eq!(span.file, "test.v3");
    assert_eq!(span.byte_start, 8, "span points at the `x` reference");
    assert_eq!(span.byte_end, 9);
}

#[test]
fn test_arity_mismatch_span_points_at_call_site() {
    // `fn f(a: Int) -> Int = a\nlet x = f(1, 2)`
    //   l  e  t     x  =     f  (  1  ,     2  )
    //  24 25 26 27 28 29 30 31 32 33 34 35 36 37
    // The call expression `f(1, 2)` starts at byte 32 and ends at 39.
    let src = "fn f(a: Int) -> Int = a\nlet x = f(1, 2)";
    let dag = compile_any(src, "test.v3");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
    let diag = dag
        .diagnostics()
        .get(bind_x.value)
        .expect("arity mismatch recorded");
    let span = diag.span();
    assert_eq!(span.file, "test.v3");
    // The diagnostic points at the Transform node's span — the
    // whole `f(1, 2)` call.
    assert!(
        span.byte_start >= 32 && span.byte_end > span.byte_start,
        "span covers the call site, got [{}, {})",
        span.byte_start,
        span.byte_end
    );
    assert!(
        span.byte_end <= src.len() as u32,
        "span is within source bounds"
    );
}

#[test]
fn test_unknown_function_span_points_at_call_site() {
    // `let x = unknown_fn(1)` — the ResolveError span should
    // cover the call, not be a synthetic `<inferred>` span.
    let src = "let x = unknown_fn(1)";
    let dag = compile_any(src, "test.v3");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
    let diag = dag
        .diagnostics()
        .get(bind_x.value)
        .expect("resolve error recorded");
    let span = diag.span();
    assert_eq!(span.file, "test.v3");
    assert_ne!(
        span.file, "<inferred>",
        "span should come from the source, not a synthetic fallback"
    );
    assert!(span.byte_end > span.byte_start);
    assert!(span.byte_end <= src.len() as u32);
}

#[test]
fn test_provenance_lens_branch_origin() {
    // When a Bind's value port is produced by a Branch, the
    // provenance lens reports Origin::Selected pointing at the
    // Branch node. The lens reads only produced_by and the
    // producer's behavior kind — no reconstruction.
    let src = "let x = if 1 > 0 then 42 else 0";
    let dag = compile_to_dag(src, "test.v3").expect("compiles");
    let lens = ProvenanceLens::new(&dag);

    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");

    match lens.origin_of(bind_x.value) {
        Origin::Selected { by } => {
            assert!(
                dag.node(by).as_branch().is_some(),
                "Selected origin points at a Branch node"
            );
        }
        other => panic!("expected Origin::Selected for Branch output, got {other:?}"),
    }
}

#[test]
fn test_provenance_lens_loop_origin() {
    // A recursive function's Bind.value port is produced by a
    // Loop node (the bounded-recursion wrapper). The provenance
    // lens reports Origin::Accumulated pointing at the Loop node.
    let src = "fn count(n: Int) -> Int = if n == 0 then 0 else n + count(n - 1)\nlet answer = count(3)";
    let dag = compile_to_dag(src, "test.v3").expect("compiles");
    let lens = ProvenanceLens::new(&dag);

    let count_bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "count")
        .expect("Bind(count) exists");

    match lens.origin_of(count_bind.value) {
        Origin::Accumulated { by } => {
            assert!(
                dag.node(by).as_loop().is_some(),
                "Accumulated origin points at a Loop node"
            );
        }
        other => panic!("expected Origin::Accumulated for Loop output, got {other:?}"),
    }
}

#[test]
fn test_composition_nested_let_bindings() {
    // Three-deep let chain where each binding references the previous.
    let src = "let a = 1\nlet b = a + 1\nlet c = b + a";
    let dag = compile_to_dag(src, "test.v3").expect("compiles");
    // Expected shape: Value(1) + Bind(a)
    //                 + Transform(Add) + Bind(b)     (uses a)
    //                 + Transform(Add) + Bind(c)     (uses b, a)
    let c_bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "c")
        .expect("Bind(c) exists");
    assert_eq!(
        dag.port(c_bind.value).value_type(),
        Some(&TypeShape::Primitive(Prim::Int)),
        "c is typed Int through composition"
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn test_composition_nested_if_expressions() {
    // Nested if in the else branch: `if a then b else (if c then d else e)`.
    let src = "let r = if 1 > 0 then 10 else if 2 > 0 then 20 else 30";
    let dag = compile_to_dag(src, "test.v3").expect("compiles");
    let bind_r = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "r")
        .expect("Bind(r) exists");
    let outer_branch_id = dag
        .port(bind_r.value)
        .produced_by
        .expect("producer exists");
    let outer_branch = dag
        .node(outer_branch_id)
        .as_branch()
        .expect("producer is a Branch");
    assert_eq!(outer_branch.paths.len(), 2);

    // The else path should itself be a Branch (the nested if).
    let else_path = &outer_branch.paths[1];
    let inner = dag
        .node(else_path.body)
        .as_branch()
        .expect("else path body is a nested Branch");
    assert_eq!(inner.paths.len(), 2);

    assert_eq!(
        dag.port(bind_r.value).value_type(),
        Some(&TypeShape::Primitive(Prim::Int)),
        "nested-if unification gives Int"
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn test_composition_if_inside_function_call() {
    // If-expression as a function-call argument.
    let src = "fn f(x: Int) -> Int = x + 1\nlet y = f(if 1 > 0 then 10 else 20)";
    let dag = compile_to_dag(src, "test.v3").expect("compiles");
    let bind_y = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "y")
        .expect("Bind(y) exists");
    let call_id = dag
        .port(bind_y.value)
        .produced_by
        .expect("producer exists");
    let call = dag
        .node(call_id)
        .as_transform()
        .expect("producer is a Transform");
    assert_target_name(&dag, call.target, "f");
    assert_eq!(call.inputs.len(), 1);

    // The argument is a Branch output.
    let arg_producer = dag
        .port(call.inputs[0])
        .produced_by
        .expect("arg has producer");
    assert!(
        dag.node(arg_producer).as_branch().is_some(),
        "function argument is an if-expression lowered to a Branch"
    );

    assert_eq!(
        dag.port(bind_y.value).value_type(),
        Some(&TypeShape::Primitive(Prim::Int)),
        "f returns Int regardless of which path the if chose"
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn test_composition_two_functions_later_calls_earlier() {
    // Function `g` defined after `f` and calls `f`. Multiple
    // functions in sequence with forward dependency from g into f.
    let src = "fn f(x: Int) -> Int = x + 1\nfn g(y: Int) -> Int = f(y) + 1\nlet r = g(5)";
    let dag = compile_to_dag(src, "test.v3").expect("compiles");
    let bind_r = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "r")
        .expect("Bind(r) exists");
    let g_call_id = dag
        .port(bind_r.value)
        .produced_by
        .expect("producer exists");
    let g_call = dag
        .node(g_call_id)
        .as_transform()
        .expect("producer is a Transform");
    assert_target_name(&dag, g_call.target, "g");

    // Both f and g should be registered as Bind nodes with
    // non-empty params (function definitions).
    let f_bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "f")
        .expect("Bind(f) exists");
    let g_bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "g")
        .expect("Bind(g) exists");
    assert_eq!(f_bind.params.len(), 1);
    assert_eq!(g_bind.params.len(), 1);

    assert_eq!(
        dag.port(bind_r.value).value_type(),
        Some(&TypeShape::Primitive(Prim::Int)),
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn test_composition_branch_with_function_calls_in_both_paths() {
    // Both paths of an if contain function calls — exercise the
    // Branch path-unification with non-trivial path bodies.
    let src = "fn f(x: Int) -> Int = x\nfn g(x: Int) -> Int = x + 1\nlet r = if 1 > 0 then f(10) else g(20)";
    let dag = compile_to_dag(src, "test.v3").expect("compiles");
    let bind_r = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "r")
        .expect("Bind(r) exists");
    let branch_id = dag
        .port(bind_r.value)
        .produced_by
        .expect("producer exists");
    let branch = dag
        .node(branch_id)
        .as_branch()
        .expect("producer is a Branch");
    assert_eq!(branch.paths.len(), 2);

    // Each path body is a Transform (the function calls).
    for path in &branch.paths {
        let body = dag.node(path.body);
        assert!(
            body.as_transform().is_some(),
            "path body is a Transform (function call)"
        );
    }
    assert_eq!(
        dag.port(bind_r.value).value_type(),
        Some(&TypeShape::Primitive(Prim::Int)),
    );
    assert!(dag.diagnostics().is_empty());
}

// ════════════════════════════════════════════════════════════════
// M0.8 — Reviewer regression tests for the 5 blockers caught in
// external review. Each test corresponds to one bug class that
// Tests 1-5 did not exercise.
// ════════════════════════════════════════════════════════════════

#[test]
fn reviewer_type_annotation_does_not_override_unresolved() {
    // `let x: Bool = y` where `y` is undefined.
    //
    // Sequence: (1) lowering the `y` reference marks its placeholder
    // port Unresolved with a ResolveError. (2) The type annotation
    // would normally call set_port_type(value_port, Bool), but
    // set_port_type checks for Unresolved state and refuses to
    // transition out. (3) End state: port stays Unresolved, the
    // biconditional holds.
    //
    // This test documents the M0.6 structural fix and serves as a
    // regression guard against a future refactor that forgets the
    // Unresolved-check in set_port_type.
    let dag = compile_any("let x: Bool = y", "test.v3");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
    assert!(
        matches!(dag.port(bind_x.value).state(), PortState::Unresolved),
        "value port stays Unresolved even after the Bool annotation attempt"
    );
    assert!(
        dag.diagnostics().contains(bind_x.value),
        "diagnostic entry exists (from the original ResolveError)"
    );
}

#[test]
fn reviewer_non_bool_branch_condition_is_rejected() {
    // `if 1 then 2 else 3` — the condition is Int, not Bool. The
    // Branch input Bool check must fire and mark the Branch output
    // Unresolved with a TypeMismatch diagnostic.
    let dag = compile_any("let x = if 1 then 2 else 3", "test.v3");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
    assert!(
        matches!(dag.port(bind_x.value).state(), PortState::Unresolved),
        "non-Bool branch condition makes the Branch output Unresolved"
    );
    let diag = dag
        .diagnostics()
        .get(bind_x.value)
        .expect("diagnostic recorded");
    match diag {
        v3_compiler::Diagnostic::TypeMismatch {
            expected, actual, ..
        } => {
            assert_eq!(*expected, TypeShape::Primitive(Prim::Bool));
            assert_eq!(*actual, TypeShape::Primitive(Prim::Int));
        }
        other => panic!("expected TypeMismatch Bool/Int, got {other:?}"),
    }
}

#[test]
fn reviewer_call_site_rejects_function_with_invalid_body() {
    // `fn f(a: Int) -> Bool = 1` declares return type Bool but the
    // body is `1` (Int). The apply-level conflict check catches the
    // body/declaration mismatch and marks f's value port Unresolved.
    //
    // Then `let x = f(1)` looks up f. The Bind-state check in
    // Transform decide() sees f.value is Unresolved and refuses to
    // trust the registered signature. x's value port also becomes
    // Unresolved.
    //
    // Before the M0.8 fix, the signature registry was consulted
    // without checking the Bind state, so f(1) would type as Bool
    // even though the function body was wrong. The v2 disease
    // recurring in v3.
    let dag = compile_any("fn f(a: Int) -> Bool = 1\nlet x = f(1)", "test.v3");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
    assert!(
        matches!(dag.port(bind_x.value).state(), PortState::Unresolved),
        "call site rejects function whose body conflicts with signature"
    );
    let f_bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "f")
        .expect("Bind(f) exists");
    assert!(
        matches!(dag.port(f_bind.value).state(), PortState::Unresolved),
        "the function's own value port is Unresolved from the body/declaration conflict"
    );
}

#[test]
fn reviewer_zero_arg_recursion_is_rejected() {
    // A function with no parameters that calls itself has no
    // termination measure. It must be rejected at lower time, not
    // silently accepted as an infinite loop.
    //
    // Note: `loop` is not a Rust keyword inside identifier context,
    // and the v3 tokenizer recognizes it as a plain identifier.
    let dag = compile_any("fn endless() -> Int = endless()", "test.v3");
    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "endless")
        .expect("Bind(endless) exists");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "zero-arg recursion is rejected"
    );
}

#[test]
fn reviewer_non_decreasing_recursion_is_rejected() {
    // `fn f(n: Int) -> Int = f(n)` recurses with the same argument.
    // The descent check requires `first_param - <positive int>` and
    // this fails (first arg is `n`, not `n - k`). Must be rejected.
    let dag = compile_any("fn f(n: Int) -> Int = f(n)", "test.v3");
    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "f")
        .expect("Bind(f) exists");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "non-decreasing recursion is rejected"
    );
}

#[test]
fn reviewer_descent_on_growing_arg_is_rejected() {
    // `fn f(n: Int) -> Int = f(n + 1)` grows the argument. The
    // descent check only accepts `n - <positive>`, not `n + <...>`.
    // Conservative: reject anything that's not obviously decreasing.
    let dag = compile_any("fn f(n: Int) -> Int = f(n + 1)", "test.v3");
    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "f")
        .expect("Bind(f) exists");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "`n + 1` as recursive argument is rejected"
    );
}

#[test]
fn test_bool_literal_fails_when_int_expected() {
    // Lane B follow-up: failure-mode for bool literals. The
    // annotation direction (Int expected, Bool actual) exercises
    // the apply-level conflict check opposite from
    // test_type_mismatch_produces_diagnostic_entry (Bool expected,
    // Int actual).
    let dag = compile_any("let x: Int = true", "test.v3");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
    assert!(
        matches!(dag.port(bind_x.value).state(), PortState::Unresolved),
        "Bool literal where Int is expected produces Unresolved"
    );
    let diag = dag
        .diagnostics()
        .get(bind_x.value)
        .expect("diagnostic recorded");
    match diag {
        v3_compiler::Diagnostic::TypeMismatch {
            expected, actual, ..
        } => {
            assert_eq!(*expected, TypeShape::Primitive(Prim::Int));
            assert_eq!(*actual, TypeShape::Primitive(Prim::Bool));
        }
        other => panic!("expected TypeMismatch Int/Bool, got {other:?}"),
    }
}

#[test]
fn test_string_literal_fails_when_int_expected() {
    // Lane B follow-up: String actual, Int expected. Covers the
    // third primitive pair that the existing type_mismatch tests
    // didn't exercise.
    let dag = compile_any("let x: Int = \"hello\"", "test.v3");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
    assert!(
        matches!(dag.port(bind_x.value).state(), PortState::Unresolved),
        "String literal where Int is expected produces Unresolved"
    );
    let diag = dag
        .diagnostics()
        .get(bind_x.value)
        .expect("diagnostic recorded");
    match diag {
        v3_compiler::Diagnostic::TypeMismatch {
            expected, actual, ..
        } => {
            assert_eq!(*expected, TypeShape::Primitive(Prim::Int));
            assert_eq!(*actual, TypeShape::Primitive(Prim::String));
        }
        other => panic!("expected TypeMismatch Int/String, got {other:?}"),
    }
}

#[test]
fn test_string_branch_condition_is_rejected() {
    // Lane B follow-up: non-Bool branch condition that is String
    // rather than Int. Exercises the Branch Bool check on the third
    // primitive type — reviewer_non_bool_branch_condition_is_rejected
    // only covered Int. Together these prove the check rejects ANY
    // non-Bool, not just Int specifically.
    let dag = compile_any("let x = if \"foo\" then 1 else 2", "test.v3");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
    assert!(
        matches!(dag.port(bind_x.value).state(), PortState::Unresolved),
        "String branch condition is rejected"
    );
    let diag = dag
        .diagnostics()
        .get(bind_x.value)
        .expect("diagnostic recorded");
    match diag {
        v3_compiler::Diagnostic::TypeMismatch {
            expected, actual, ..
        } => {
            assert_eq!(*expected, TypeShape::Primitive(Prim::Bool));
            assert_eq!(*actual, TypeShape::Primitive(Prim::String));
        }
        other => panic!("expected TypeMismatch Bool/String, got {other:?}"),
    }
}

#[test]
fn reviewer_unknown_type_name_is_rejected() {
    // `let x: NotARealType = 1` — the type annotation references
    // an unknown name. The lower_type Result path must surface a
    // ResolveError and mark the value port Unresolved, not silently
    // default to Int.
    //
    // Before the M0.8 fix, lower_type had a silent `_ => Int`
    // default, so this program compiled as if it said `let x: Int
    // = 1` with no diagnostic.
    let dag = compile_any("let x: NotARealType = 1", "test.v3");
    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "unknown type name on let produces Unresolved port"
    );
    let diag = dag
        .diagnostics()
        .get(bind.value)
        .expect("diagnostic recorded");
    assert!(
        matches!(diag, v3_compiler::Diagnostic::ResolveError { .. }),
        "diagnostic is a ResolveError, got {diag:?}"
    );
}

// ════════════════════════════════════════════════════════════════
// Depth lens — second observational lens. Validates the success
// bar empirically: adding a new read-only analysis should cost
// tens of lines and zero substrate modifications. lens_depth.rs
// is 66 lines (incl. doc comment) and touches no substrate file.
// ════════════════════════════════════════════════════════════════

#[test]
fn test_depth_lens_let_binding() {
    // `let x = 1 + 2`
    //   Value(1) depth 0, Value(2) depth 0
    //   Add.output depth = 1 + max(0, 0) = 1
    //   Bind(x).value = Add.output, depth 1
    let dag = compile_to_dag("let x = 1 + 2", "test.v3").expect("compiles");
    let lens = DepthLens::new(&dag);
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .unwrap();
    assert_eq!(lens.depth_of(bind_x.value), 1);
}

#[test]
fn test_depth_lens_nested_arithmetic() {
    // `let z = 1 + 2 + 3`
    //   left-associative: ((1 + 2) + 3)
    //   inner Add depth 1, Value(3) depth 0
    //   outer Add depth = 1 + max(1, 0) = 2
    let dag = compile_to_dag("let z = 1 + 2 + 3", "test.v3").expect("compiles");
    let lens = DepthLens::new(&dag);
    let bind_z = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "z")
        .unwrap();
    assert_eq!(lens.depth_of(bind_z.value), 2);
}

#[test]
fn test_depth_lens_branch_takes_max_of_paths_and_condition() {
    // `let r = if 1 > 0 then 10 else 20 + 30`
    //   condition: Value(1), Value(0) -> Gt, depth 1
    //   then:      Value(10), depth 0
    //   else:      Value(20), Value(30) -> Add, depth 1
    //   Branch depth = 1 + max(cond_depth, max(paths_depths))
    //               = 1 + max(1, max(0, 1)) = 1 + 1 = 2
    let dag = compile_to_dag(
        "let r = if 1 > 0 then 10 else 20 + 30",
        "test.v3",
    )
    .expect("compiles");
    let lens = DepthLens::new(&dag);
    let bind_r = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "r")
        .unwrap();
    assert_eq!(lens.depth_of(bind_r.value), 2);
}

// ════════════════════════════════════════════════════════════════
// M0.10 — recursive body/signature reconciliation. The M0.8 call-
// site Bind-state check handles non-recursive function body
// mismatches correctly, but recursive functions had a second bug:
// Loop.output was pre-seeded with the declared return type AND the
// body's actual return type was computed on a separate port
// (body_return_port). Nothing reconciled the two, so call sites
// read the pre-seeded declared type and trusted it even when the
// body disagreed.
// ════════════════════════════════════════════════════════════════

#[test]
fn test_recursive_function_with_wrong_body_type_is_rejected() {
    // `fn bad(n: Int) -> Bool = if n == 0 then 0 else bad(n - 1)`
    //
    // The declaration says Bool, but the body's then-path produces
    // Int (Value(0)). This is the recursive analogue of
    // `reviewer_call_site_rejects_function_with_invalid_body`
    // which only covered non-recursive functions.
    //
    // Expected: both `bad`'s value port AND the call site `x` must
    // end up Unresolved after the fix. Before the fix, loop_output
    // stays Resolved(Bool) from the pre-seed, so x gets Bool and
    // the body mismatch is invisible to consumers.
    let src = "fn bad(n: Int) -> Bool = if n == 0 then 0 else bad(n - 1)\nlet x = bad(5)";
    let dag = compile_any(src, "test.v3");
    let bad_bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "bad")
        .expect("Bind(bad) must exist");
    assert!(
        matches!(dag.port(bad_bind.value).state(), PortState::Unresolved),
        "bad's value port is Unresolved because the body doesn't match the declared return type; got {:?}",
        dag.port(bad_bind.value).state()
    );
    let x_bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) must exist");
    assert!(
        matches!(dag.port(x_bind.value).state(), PortState::Unresolved),
        "call site x is Unresolved because bad is invalid; got {:?}",
        dag.port(x_bind.value).state()
    );
}

// ════════════════════════════════════════════════════════════════
// M0.11 — mutual recursion rejection. `is_recursive` only finds
// direct self-calls, so `fn even(n) = odd(n-1)` paired with
// `fn odd(n) = even(n-1)` neither hits the Loop wrap path nor
// gets rejected. The fix adds a pre-lowering pass that computes
// the call graph, finds SCCs, and rejects any function in an SCC
// of size > 1 with a specific "mutual recursion not yet supported"
// diagnostic.
// ════════════════════════════════════════════════════════════════

#[test]
fn test_mutual_recursion_is_rejected() {
    let src = "fn even(n: Int) -> Bool = if n == 0 then true else odd(n - 1)\nfn odd(n: Int) -> Bool = if n == 0 then false else even(n - 1)";
    let dag = compile_any(src, "test.v3");
    let even_bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "even")
        .expect("Bind(even) must exist");
    assert!(
        matches!(dag.port(even_bind.value).state(), PortState::Unresolved),
        "even's value port is Unresolved because mutual recursion is not supported; got {:?}",
        dag.port(even_bind.value).state()
    );
    let odd_bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "odd")
        .expect("Bind(odd) must exist");
    assert!(
        matches!(dag.port(odd_bind.value).state(), PortState::Unresolved),
        "odd's value port is Unresolved because mutual recursion is not supported; got {:?}",
        dag.port(odd_bind.value).state()
    );
    // The diagnostic should be specific: a ResolveError whose name
    // field mentions "mutual recursion", not a generic
    // "(inference did not resolve this port)" post-sweep fallback.
    let diag = dag
        .diagnostics()
        .get(even_bind.value)
        .expect("diagnostic recorded for even");
    match diag {
        v3_compiler::Diagnostic::ResolveError { name, .. } => {
            assert!(
                name.contains("mutual recursion"),
                "diagnostic should mention mutual recursion; got `{name}`"
            );
        }
        other => panic!("expected ResolveError, got {other:?}"),
    }
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
        // Bool + string literals (Lane B)
        "let x = true",
        "let x = false",
        "let x = \"hello\"",
        "let x = if true then 1 else 2",
        // Error paths
        "let x: Bool = 1",
        "let y = x\nlet x = 1",
        "fn f(a: Int) -> Int = a\nlet x = f(1, 2)",
        "let x = unknown_fn(1)",
        // M0.7 Lane A: composition stress
        "let a = 1\nlet b = a + 1\nlet c = b + a",
        "let r = if 1 > 0 then 10 else if 2 > 0 then 20 else 30",
        "fn f(x: Int) -> Int = x + 1\nlet y = f(if 1 > 0 then 10 else 20)",
        "fn f(x: Int) -> Int = x + 1\nfn g(y: Int) -> Int = f(y) + 1\nlet r = g(5)",
        "fn f(x: Int) -> Int = x\nfn g(x: Int) -> Int = x + 1\nlet r = if 1 > 0 then f(10) else g(20)",
        // M0.8: reviewer regression sources
        "let x: Bool = y",
        "let x = if 1 then 2 else 3",
        "fn f(a: Int) -> Bool = 1\nlet x = f(1)",
        "fn endless() -> Int = endless()",
        "fn f(n: Int) -> Int = f(n)",
        "fn f(n: Int) -> Int = f(n + 1)",
        "let x: NotARealType = 1",
        // Lane B follow-up: failure-mode primitive pairs
        "let x: Int = true",
        "let x: Int = \"hello\"",
        "let x = if \"foo\" then 1 else 2",
        // M0.10: recursive function body/signature mismatch
        "fn bad(n: Int) -> Bool = if n == 0 then 0 else bad(n - 1)\nlet x = bad(5)",
        // M0.11: mutual recursion
        "fn even(n: Int) -> Bool = if n == 0 then true else odd(n - 1)\nfn odd(n: Int) -> Bool = if n == 0 then false else even(n - 1)",
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
