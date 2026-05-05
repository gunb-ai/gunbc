//! Prereq-X (call-on-field-access) ratchet — post E6-G0b X1.a slice.
//!
//! Records the parser+lowerer state after Prereq-X1.a (static
//! call-on-field-access via `data` binding head) landed. The audit
//! at `docs/design-prereq-x-ho-field-call.md` (PR #1264) decomposes
//! Prereq-X into X1 (call-on-field-access), X2 (call-on-Var if not
//! subsumed by X1), and X3 (explicit block expressions inside `=`
//! bodies); the X1.a / X1.b split is named in
//! `docs/briefs/r3-pr-e6-g0b-x1a-static-field-call-worker.md`.
//!
//! State after this slice:
//!
//! - X1.a: `data v: WrapFn = { f: double }; v.f(x)` parses **and
//!   lowers** to a static-callable invocation.
//! - X1.b: `fn invoke(w: WrapFn, x: Int) -> Int = w.f(x)` **parses**
//!   (the parser surface accepts call-on-field-access) but **lowers**
//!   to a typed `ResolveError` naming the X1.b prerequisite. Parser
//!   parse-error ratchet is replaced by a lowering-diagnostic ratchet.
//! - X3: brace-bodied block expression inside `=` body remains
//!   blocked at parse time.

use crate::common::{cached_compile_any, cached_compile_to_dag};
use v3_compiler::dag::{Behavior, TransformTarget};
use v3_compiler::diagnostics::Diagnostic;
use v3_compiler::{parse_for_test, tokenize_for_test};

/// Control: the `type Wrapper { f: fn(Int) -> Int }` declaration on its
/// own parses cleanly. Confirms the X1/X3 fixtures' parse failures
/// originate at the `w.f(x)` / `{ let g = w.f; g(x) }` call site, not at
/// the type declaration.
#[test]
fn control_arrow_typed_field_decl_parses() {
    let src = "type Wrapper { f: fn(Int) -> Int }\n";
    let tokens = tokenize_for_test(src, "control.v3").expect("tokenize");
    parse_for_test(&tokens, "control.v3").expect(
        "Arrow-typed field declaration must parse cleanly so X1/X3 isolate the call-site gap.",
    );
}

/// X1.a positive: `data v: WrapFn = { f: double }; ... v.f(x)` parses
/// and lowers to a static `TransformTarget::Callable(decl_id_of_double)`
/// transform. Asserted by structural inspection of the lowered Dag, not
/// by emit-roundtrip output (E6-G0c scope).
#[test]
fn x1a_static_data_field_call_lowers_to_callable() {
    let src = r#"
type Wrapper { f: fn(Int) -> Int }

fn double(n: Int) -> Int = n + n

data wrap: Wrapper = { f: double }

fn invoke(x: Int) -> Int = wrap.f(x)
"#;
    let dag = cached_compile_to_dag(src, "x1a_positive.v3");
    let double_decl_id = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("double"))
        .map(|d| d.id)
        .expect("`double` declaration");
    let saw_callable_to_double = dag.nodes().iter().any(|node| match node {
        Behavior::Transform(t) => matches!(
            &t.target,
            TransformTarget::Callable(id) if *id == double_decl_id
        ),
        _ => false,
    });
    assert!(
        saw_callable_to_double,
        "X1.a should lower `wrap.f(x)` to a TransformTarget::Callable pointing at `double`'s decl_id; got Dag without that target"
    );
}

/// X1.a negative: `data v: { x: Int } = { x: 5 }; v.x(7)` lowers to a
/// typed `ResolveError` naming the field as non-Arrow / non-callable —
/// not a parse error.
#[test]
fn x1a_non_arrow_field_call_diagnostic() {
    let src = r#"
type Holder { x: Int }

data holder: Holder = { x: 5 }

fn r() -> Int = holder.x(7)
"#;
    let dag = cached_compile_any(src, "x1a_non_arrow.v3");
    let saw_resolve_diagnostic = dag.diagnostics().iter().any(|(_, d)| match d {
        Diagnostic::ResolveError { name, .. } => {
            name.contains("does not resolve to a callable function reference")
                || name.contains("non-Arrow")
                || name.contains("FieldValue::Reference")
        }
        _ => false,
    });
    assert!(
        saw_resolve_diagnostic,
        "non-Arrow X1.a callee must produce a typed ResolveError naming the non-callable leaf"
    );
}

/// X1.b: parameter-callee dispatch through `<param>.<field>(args)`
/// **parses** (after X1.a landed the surface grammar) but **lowers**
/// to a typed `ResolveError` naming the X1.b prerequisite (runtime-callee
/// substrate / `TransformDispatch::Indirect`). Parser parse-error
/// ratchet retired; lowering-diagnostic ratchet replaces it.
#[test]
fn x1b_parameter_field_call_blocked_at_lowering() {
    let src = r#"
type Wrapper { f: fn(Int) -> Int }

fn invoke(w: Wrapper, x: Int) -> Int = w.f(x)
"#;
    // Parsing now succeeds; the diagnostic surfaces during lowering.
    let dag = cached_compile_any(src, "x1b.v3");
    let saw_x1b_diagnostic = dag.diagnostics().iter().any(|(_, d)| match d {
        Diagnostic::ResolveError { name, .. } => {
            name.contains("Prereq-X1.b") || name.contains("parameter")
        }
        _ => false,
    });
    assert!(
        saw_x1b_diagnostic,
        "X1.b parameter-callee must lower-fail with a typed ResolveError naming the X1.b prerequisite"
    );
}

/// X1.a P4 regression: mutual recursion routed through a `data`
/// binding indirection must contribute edges to the cluster
/// descent gate. Without resolving PathCall through symbols+ValueBody,
/// `data fns: Fns = { a: f, b: g }; fn f(x) = fns.b(x); fn g(x) =
/// fns.a(x)` would parse + lower without engaging the strict-descent
/// check (INVARIANTS P4 hole). This test pins the gate firing.
#[test]
fn x1a_mutual_recursion_via_data_binding_engages_descent_gate() {
    // Note: `data fns` must precede `fn f`/`fn g` so its
    // `ValueBody::Structural` is populated by the time the function
    // bodies are lowered and recursion analysis runs. The pre-pass
    // collects all top-level decls into `symbols` regardless of
    // order, but `value_body` is only populated when each `Data`
    // item is lowered in source order.
    let src = r#"
type Fns { a: fn(Int) -> Int, b: fn(Int) -> Int }

data fns: Fns = { a: f, b: g }

fn f(x: Int) -> Int = fns.b(x)
fn g(x: Int) -> Int = fns.a(x)
"#;
    let dag = cached_compile_any(src, "x1a_mutual_via_data.v3");
    // Both `f` and `g` are mutually recursive via the `fns` data
    // binding indirection. The cluster descent gate must fire because
    // neither body strictly decreases its first argument; we expect a
    // typed ResolveError or descent diagnostic mentioning recursion or
    // descent. The exact wording is determined by the existing
    // `cannot terminate` / descent-fail-closed diagnostic family;
    // assert at least one such diagnostic surfaces.
    let saw_decidability_diagnostic = dag.diagnostics().iter().any(|(_, d)| match d {
        Diagnostic::ResolveError { name, .. } => {
            name.contains("recursive")
                || name.contains("terminate")
                || name.contains("descent")
                || name.contains("cannot")
        }
        _ => false,
    });
    assert!(
        saw_decidability_diagnostic,
        "Mutual recursion via data-binding indirection must engage the decidability gate; got Dag without any termination/descent diagnostic. This regression-locks the X1.a P4 fix in is_recursive / descent_provable / ClusterDescentChecker / collect_recursive_callees."
    );
}

/// X3: brace-bodied block expression inside `=` body, with a `let` head.
/// Required to factor `let g = w.f; g(x)` out of a SingleRoot fold.
/// **Unchanged by the X1.a slice** — block-expression bodies still fail
/// at parse time.
#[test]
fn x3_brace_block_with_let_head_blocked() {
    let src = r#"
type Wrapper { f: fn(Int) -> Int }

fn invoke(w: Wrapper, x: Int) -> Int = { let g = w.f; g(x) }
"#;
    let tokens = tokenize_for_test(src, "x3.v3").expect("tokenize");
    let err = parse_for_test(&tokens, "x3.v3").expect_err(
        "Prereq-X3 still blocks `= { let ...; ... }` — if this test panics, retire this ratchet.",
    );
    assert!(
        err.message().contains("LParen") || err.message().contains("KwLet"),
        "X3 diagnostic shape changed; verify against #1264 audit. Got: {}",
        err.message()
    );
}

/// Post-G0c executable ratchet: the X1.a static field-call lowered in
/// PR #1699 actually executes through the public evaluator boundary
/// after PR #1715 landed `TransformTarget::Callable` execution. Ties
/// the parse → lower → execute pipeline together so any future
/// regression in any of the three layers fails this single ratchet.
///
/// Compiles `data wrap: Wrapper = { f: double }; fn invoke(x: Int) ->
/// Int = wrap.f(x)`, finds `invoke` honestly through
/// `Dag::declaration_by_name` + `TypeConnective::Arrow.body`,
/// pre-binds the parameter port to `Int(21)` in the caller frame, and
/// evaluates the bind through `v3_compiler::evaluator::evaluate_body`.
/// Asserts `Value::LiteralValue(Int(42))`.
#[test]
fn x1a_static_data_field_call_executes_through_public_evaluator() {
    use v3_compiler::dag::{ArrowBody, LiteralBits, TypeConnective};
    use v3_compiler::evaluator::{
        evaluate_body, EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder, Value,
    };

    let src = r#"
type Wrapper { f: fn(Int) -> Int }

fn double(n: Int) -> Int = n + n

data wrap: Wrapper = { f: double }

fn invoke(x: Int) -> Int = wrap.f(x)
"#;
    let dag = cached_compile_to_dag(src, "x1a_eval.v3");
    let invoke_decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("invoke"))
        .expect("`invoke` declaration");
    let TypeConnective::Arrow { body, .. } = &invoke_decl.connective else {
        panic!(
            "`invoke` should have Arrow connective; got {:?}",
            invoke_decl.connective
        );
    };
    let ArrowBody::UserDefined(bind_id) = body else {
        panic!("`invoke` should have UserDefined body; got {body:?}");
    };
    let bind_node_id = bind_id.node_id();
    let Behavior::Bind(bind) = dag.node(bind_node_id) else {
        panic!("BindNodeId must point at a Bind node");
    };
    assert_eq!(
        bind.params.len(),
        1,
        "`invoke` should have exactly one parameter port"
    );
    let x_port = bind.params[0];

    let caller_frame =
        EvalFrame::from_bindings([(x_port, Value::LiteralValue(LiteralBits::Int(21)))])
            .expect("caller frame");
    let mut state = EvalStateStack::with_root_frame(caller_frame);
    let strategy = EvalStrategy::ApplicativeOrder {
        input_order: InputEvaluationOrder::LeftFirst,
    };

    let value = evaluate_body(&dag, bind_node_id, &mut state, strategy)
        .expect("X1.a static field-call should execute through the public evaluator");

    assert_eq!(
        value,
        Value::LiteralValue(LiteralBits::Int(42)),
        "wrap.f(21) should resolve to double(21) = 42 via TransformTarget::Callable execution"
    );
}
