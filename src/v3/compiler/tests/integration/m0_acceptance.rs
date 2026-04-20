//! **Layer:** integration
//!
//! Minimal M0 acceptance receipts.
//!
//! This file intentionally keeps only the smallest set of still-distinct
//! milestone-0 contracts:
//! - straight-line lowering and inference
//! - branch lowering and type unification
//! - bounded self-recursion lowering
//! - fail-closed semantic diagnostics for representative error classes
//! - post-sweep port-state invariants

use crate::common::cached_compile_to_dag;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    AtomPayload, Behavior, Dag, LiteralBits, LoopBound, PortState, TransformTarget, TypeConnective,
};
use v3_compiler::types::TypeShape;
use v3_compiler::{CompileError, Diagnostic};

fn primitive_shape(dag: &Dag, name: &str) -> TypeShape {
    TypeShape::new(
        dag.declaration_by_name(name)
            .unwrap_or_else(|| panic!("primitive `{name}` missing from bootstrap"))
            .id,
    )
}

fn assert_target_name(dag: &Dag, target: &TransformTarget, expected: &str) {
    let actual = match target {
        TransformTarget::Callable(id) => {
            let decl = dag.declaration(*id);
            match &decl.connective {
                TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name)) => Some(name.clone()),
                TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
                | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                    dag.declaration(*next).name.clone()
                }
                _ => decl.name.clone(),
            }
        }
        TransformTarget::FieldProject { field_label, .. } => Some(format!(".{field_label}")),
        TransformTarget::Operator(op_kind) => Some(v3_compiler::operators::symbol(*op_kind)),
    };
    assert_eq!(actual.as_deref(), Some(expected));
}

fn compile_any(src: &str, file: &str) -> Dag {
    match compile_to_dag(src, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}

fn bind_named<'a>(dag: &'a Dag, name: &str) -> &'a v3_compiler::dag::BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
}

#[test]
fn let_binding_lowers_to_a_typed_add() {
    let dag = cached_compile_to_dag("let x = 1 + 2", "m0_let.v3");
    let bind = bind_named(&dag, "x");

    let add = dag
        .node(dag.port(bind.value).produced_by.expect("bind producer"))
        .as_transform()
        .expect("bind value should come from a Transform");
    assert_target_name(&dag, &add.target, "+");
    assert_eq!(add.inputs.len(), 2);

    let lhs = dag
        .node(dag.port(add.inputs[0]).produced_by.expect("lhs producer"))
        .as_value()
        .expect("lhs should be a Value");
    let rhs = dag
        .node(dag.port(add.inputs[1]).produced_by.expect("rhs producer"))
        .as_value()
        .expect("rhs should be a Value");
    assert_eq!(lhs.data, LiteralBits::Int(1));
    assert_eq!(rhs.data, LiteralBits::Int(2));
    assert_eq!(
        dag.port(bind.value).value_type(),
        Some(&primitive_shape(&dag, "Int"))
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn if_expression_lowers_to_a_typed_branch() {
    let src = "let x = 5\nlet result = if x > 0 then 1 else 2";
    let dag = cached_compile_to_dag(src, "m0_if.v3");
    let bind = bind_named(&dag, "result");

    let branch = dag
        .node(dag.port(bind.value).produced_by.expect("bind producer"))
        .as_branch()
        .expect("bind value should come from a Branch");
    assert_eq!(branch.paths.len(), 2);

    let cmp = dag
        .node(
            dag.port(branch.input)
                .produced_by
                .expect("condition producer"),
        )
        .as_transform()
        .expect("branch input should come from a Transform");
    assert_target_name(&dag, &cmp.target, ">");
    assert_eq!(
        dag.port(branch.input).value_type(),
        Some(&primitive_shape(&dag, "Bool"))
    );
    assert_eq!(
        dag.port(branch.output).value_type(),
        Some(&primitive_shape(&dag, "Int"))
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn numeric_recursion_lowers_to_a_bounded_loop() {
    let src = "fn count_down(n: Int) -> Int = if n == 0 then 0 else n + count_down(n - 1)\nlet answer = count_down(3)";
    let dag = cached_compile_to_dag(src, "m0_loop.v3");

    let count_down = bind_named(&dag, "count_down");
    assert_eq!(count_down.params.len(), 1);
    let param_port = count_down.params[0];

    let loop_node = dag
        .node(
            dag.port(count_down.value)
                .produced_by
                .expect("loop producer"),
        )
        .as_loop()
        .expect("recursive bind should lower through Loop");
    assert!(matches!(
        loop_node.bound,
        LoopBound::Cardinality { count } if count == param_port
    ));

    let answer = bind_named(&dag, "answer");
    let call = dag
        .node(dag.port(answer.value).produced_by.expect("call producer"))
        .as_transform()
        .expect("call site should lower to Transform");
    assert_target_name(&dag, &call.target, "count_down");
    assert_eq!(call.inputs.len(), 1);
    assert_eq!(
        dag.port(answer.value).value_type(),
        Some(&primitive_shape(&dag, "Int"))
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn recursive_body_signature_mismatch_poisons_the_function_and_call_site() {
    let src = "fn bad(n: Int) -> Bool = if n == 0 then 0 else bad(n - 1)\nlet x = bad(5)";
    let dag = compile_any(src, "m0_recursive_body_signature_mismatch.v3");
    let bad_bind = bind_named(&dag, "bad");
    let x_bind = bind_named(&dag, "x");

    assert!(matches!(
        dag.port(bad_bind.value).state(),
        PortState::Unresolved
    ));
    assert!(matches!(
        dag.port(x_bind.value).state(),
        PortState::Unresolved
    ));
}

#[test]
fn growing_recursive_argument_is_rejected() {
    let dag = compile_any("fn f(n: Int) -> Int = f(n + 1)", "m0_growing_recursion.v3");
    let bind = bind_named(&dag, "f");

    assert!(matches!(
        dag.port(bind.value).state(),
        PortState::Unresolved
    ));
}

#[test]
fn type_annotation_does_not_resurrect_an_unresolved_port() {
    let dag = compile_any("let x: Bool = y", "m0_annotation_after_resolve_error.v3");
    let bind = bind_named(&dag, "x");

    assert!(matches!(
        dag.port(bind.value).state(),
        PortState::Unresolved
    ));
    assert!(matches!(
        dag.diagnostics().get(bind.value),
        Some(Diagnostic::ResolveError { .. })
    ));
}

#[test]
fn compile_boundary_is_fail_closed() {
    assert!(compile_to_dag("let x = 1 + 2", "m0_ok.v3").is_ok());
    assert!(matches!(
        compile_to_dag("let x: Bool = 1", "m0_type_mismatch.v3"),
        Err(CompileError::Semantic(_))
    ));
    assert!(matches!(
        compile_to_dag("let y = x\nlet x = 1", "m0_forward_ref.v3"),
        Err(CompileError::Semantic(_))
    ));
    assert!(matches!(
        compile_to_dag("fn f(a: Int) -> Int = a\nlet x = f(1, 2)", "m0_arity.v3"),
        Err(CompileError::Semantic(_))
    ));
}

#[test]
fn post_sweep_port_state_matches_diagnostic_table() {
    let sources = [
        "let x = 1 + 2",
        "let x = 5\nlet result = if x > 0 then 1 else 2",
        "fn count_down(n: Int) -> Int = if n == 0 then 0 else n + count_down(n - 1)\nlet answer = count_down(3)",
        "let x: Bool = 1",
        "let y = x\nlet x = 1",
        "let x: Bool = y",
        "fn f(a: Int) -> Int = a\nlet x = f(1, 2)",
        "let x = if 1 then 2 else 3",
        "fn f(a: Int) -> Bool = 1\nlet x = f(1)",
        "fn f(n: Int) -> Int = f(n + 1)",
        "fn bad(n: Int) -> Bool = if n == 0 then 0 else bad(n - 1)\nlet x = bad(5)",
        "let x: NotARealType = 1",
    ];

    for src in sources {
        let dag = compile_any(src, "m0_invariant.v3");
        for port in dag.all_ports() {
            match port.state() {
                PortState::Uninferred => {
                    panic!(
                        "port {:?} is Uninferred after compile in source {src:?}",
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

#[test]
fn non_bool_branch_condition_is_rejected() {
    let dag = compile_any("let x = if 1 then 2 else 3", "m0_branch_condition.v3");
    let bind = bind_named(&dag, "x");

    assert!(matches!(
        dag.port(bind.value).state(),
        PortState::Unresolved
    ));
    match dag
        .diagnostics()
        .get(bind.value)
        .expect("diagnostic recorded")
    {
        Diagnostic::TypeMismatch {
            expected, actual, ..
        } => {
            assert_eq!(*expected, primitive_shape(&dag, "Bool"));
            assert_eq!(*actual, primitive_shape(&dag, "Int"));
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn type_mismatch_marks_the_binding_unresolved() {
    let dag = compile_any("let x: Bool = 1", "m0_type_mismatch.v3");
    let bind = bind_named(&dag, "x");
    let port = dag.port(bind.value);

    assert!(matches!(port.state(), PortState::Unresolved));
    match dag
        .diagnostics()
        .get(port.id())
        .expect("diagnostic recorded")
    {
        Diagnostic::TypeMismatch {
            expected, actual, ..
        } => {
            assert_eq!(*expected, primitive_shape(&dag, "Bool"));
            assert_eq!(*actual, primitive_shape(&dag, "Int"));
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn forward_reference_marks_the_binding_unresolved() {
    let dag = compile_any("let y = x\nlet x = 1", "m0_forward_ref.v3");
    let bind = bind_named(&dag, "y");

    assert!(matches!(
        dag.port(bind.value).state(),
        PortState::Unresolved
    ));
    assert!(matches!(
        dag.diagnostics().get(bind.value),
        Some(Diagnostic::ResolveError { .. })
    ));
}

#[test]
fn arity_mismatch_marks_the_call_unresolved() {
    let dag = compile_any("fn f(a: Int) -> Int = a\nlet x = f(1, 2)", "m0_arity.v3");
    let bind = bind_named(&dag, "x");

    assert!(matches!(
        dag.port(bind.value).state(),
        PortState::Unresolved
    ));
    assert!(matches!(
        dag.diagnostics().get(bind.value),
        Some(Diagnostic::ArityMismatch { .. })
    ));
}

#[test]
fn invalid_function_body_poisons_call_sites() {
    let dag = compile_any(
        "fn f(a: Int) -> Bool = 1\nlet x = f(1)",
        "m0_invalid_body.v3",
    );
    let f_bind = bind_named(&dag, "f");
    let x_bind = bind_named(&dag, "x");

    assert!(matches!(
        dag.port(f_bind.value).state(),
        PortState::Unresolved
    ));
    assert!(matches!(
        dag.port(x_bind.value).state(),
        PortState::Unresolved
    ));
}

#[test]
fn unknown_type_annotation_is_rejected() {
    let dag = compile_any("let x: NotARealType = 1", "m0_unknown_type.v3");
    let bind = bind_named(&dag, "x");

    assert!(matches!(
        dag.port(bind.value).state(),
        PortState::Unresolved
    ));
    assert!(matches!(
        dag.diagnostics().get(bind.value),
        Some(Diagnostic::ResolveError { .. })
    ));
}
