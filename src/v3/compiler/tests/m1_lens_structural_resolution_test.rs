// W3 — `lens_structural_resolution` acceptance tests.
//
// The lens detects leaked `ArrowBody::Pending` on named user
// Declarations. No current source path produces the violation (see
// the R13 fix at `lower.rs:2293` — `lower_fn_item`'s mutual-recursion
// arm used to emit Pending and was rewritten to emit
// `UserDefined(bind_id)` pointing at an Unresolved Bind value port).
// The lens is defense-in-depth against a future regression of that
// invariant, which means the positive test uses direct Dag injection
// via the narrow `inject_named_pending_arrow_for_test` hook rather
// than driving from source.
//
// Negative coverage drives from real source: anonymous Arrow(Pending)
// declarations (first-class `fn(Int) -> Int` type expressions) and
// named Arrow(UserDefined) declarations (ordinary user fns) should
// both leave the lens silent.

use v3_compiler::compile_to_dag;
use v3_compiler::inject_name_keyed_reference_for_test;
use v3_compiler::inject_named_pending_arrow_for_test;
use v3_compiler::lens_structural_resolution::{
    check, name_keyed_references, NameKeyedReference, UnresolvedArrowBody,
};
use v3_compiler::Dag;

fn violations(dag: &Dag) -> Vec<UnresolvedArrowBody> {
    check(dag)
}

fn name_keyed(dag: &Dag) -> Vec<NameKeyedReference> {
    name_keyed_references(dag)
}

#[test]
fn lens_flags_named_arrow_pending_injected_into_dag() {
    // Synthesize the exact shape the lens targets: a Declaration
    // with `name: Some("leaked_fn")` and `connective: Arrow { body:
    // Pending }`. `Dag::new()` provides the bootstrap declarations
    // so `int_shape` is available as a valid output-type reference.
    let mut dag = Dag::new();
    let int_output = dag.int_shape().expect("bootstrap Dag has Int").declaration;
    let decl_id = inject_named_pending_arrow_for_test(&mut dag, "leaked_fn", int_output);

    let found = violations(&dag);
    assert_eq!(
        found.len(),
        1,
        "expected exactly one violation, got: {found:?}"
    );
    assert_eq!(
        found[0].declaration, decl_id,
        "violation should point at the injected declaration"
    );
    assert_eq!(found[0].name, "leaked_fn");
}

#[test]
fn lens_silent_on_empty_bootstrap_dag() {
    // `Dag::new()` produces the bootstrap declaration set. Every
    // algebra-field Arrow in bootstrap is anonymous (`name: None`),
    // so the naming filter should silence every one. Any violation
    // here would mean the naming filter is broken — bootstrap-range
    // Pendings must not fire.
    let dag = Dag::new();
    let found = violations(&dag);
    assert!(
        found.is_empty(),
        "bootstrap Dag must produce zero violations (algebra arrows are anonymous), got: {found:?}"
    );
}

#[test]
fn lens_silent_on_named_user_defined_fn() {
    // `fn foo(x: Int) -> Int = x` lowers to a named Arrow whose
    // body is `UserDefined(bind_id)`. The lens only flags Pending,
    // so this must stay silent.
    let dag = compile_to_dag("fn foo(x: Int) -> Int = x", "user.v3").expect("compiles");
    let found = violations(&dag);
    assert!(
        found.is_empty(),
        "user fn with UserDefined body must not be flagged, got: {found:?}"
    );
}

#[test]
fn lens_silent_on_anonymous_arrow_type_expression() {
    // `type Callback { handler: fn(Int) -> Int }` creates an
    // anonymous Arrow declaration (name=None) with body=Pending as
    // the type of the `handler` field. This Pending is correct by
    // construction — the arrow is a first-class type, not a fn that
    // forgot its body — and the naming filter correctly ignores it.
    let dag =
        compile_to_dag("type Callback { handler: fn(Int) -> Int }", "user.v3").expect("compiles");
    let found = violations(&dag);
    assert!(
        found.is_empty(),
        "anonymous fn-type expressions must not be flagged, got: {found:?}"
    );
}

#[test]
fn lens_silent_on_named_type_alias_to_arrow() {
    // `type Callback = fn(Int) -> Int` lowers via `lower_type_alias`
    // → `type_to_connective` to a NAMED Declaration whose connective
    // is `Arrow { body: NoBody }`. Before the §8.11-adjacent split,
    // this site wrote `Arrow { body: Pending }` and the lens fired
    // on it as a false positive — type aliases never need body
    // patching, so a Pending here was structurally indistinguishable
    // from the R13 leak the lens watches for.
    //
    // The fix: split `Pending` (executable-fn realization-lag scaffold,
    // dissolves via the §8.11 ratchet) from `NoBody` (terminal —
    // the arrow has no executable body by construction, e.g. type
    // aliases). The lens still flags only `Pending`; `NoBody` is
    // silently excluded.
    let dag = compile_to_dag("type Callback = fn(Int) -> Int", "user.v3").expect("compiles");
    let found = violations(&dag);
    assert!(
        found.is_empty(),
        "named type-alias arrow (NoBody) must not be flagged, got: {found:?}"
    );
}

#[test]
fn lens_survives_co_existing_injected_and_compiled_declarations() {
    // Compose: a real source program + an injected named Pending on
    // top. Confirms the lens walks every declaration and flags the
    // injected one even when real declarations surround it. Guards
    // against off-by-one iteration bugs that might miss the
    // last-pushed declaration.
    let mut dag = compile_to_dag("fn good(x: Int) -> Int = x + 1", "user.v3").expect("compiles");
    let int_output = dag.int_shape().expect("Int shape").declaration;
    let leak_id = inject_named_pending_arrow_for_test(&mut dag, "leaked", int_output);
    let found = violations(&dag);
    assert_eq!(
        found.len(),
        1,
        "expected exactly one violation amid real declarations, got: {found:?}"
    );
    assert_eq!(found[0].declaration, leak_id);
    assert_eq!(found[0].name, "leaked");
}

#[test]
fn lens_flags_injected_name_keyed_reference() {
    let mut dag = Dag::new();
    let int_id = dag.int_shape().expect("bootstrap Dag has Int").declaration;
    let site_id = inject_name_keyed_reference_for_test(&mut dag, int_id);

    let found = name_keyed(&dag);
    assert_eq!(
        found.len(),
        1,
        "expected exactly one name-keyed reference, got: {found:?}"
    );
    assert_eq!(found[0].declaration, site_id);
    assert_eq!(found[0].resolved_to, int_id);
}
