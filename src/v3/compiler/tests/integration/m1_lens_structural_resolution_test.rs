// W3 — `lens_structural_resolution` integration receipts.
//
// The direct-Dag synthetic cases now live in
// `src/v3/compiler/src/lib.rs::lens_structural_resolution::tests`, where
// crate-private declaration mutation is reachable without a public shim.
// This file keeps only receipts that still need real source lowering.

use v3_compiler::compile_to_dag;
use v3_compiler::lens_structural_resolution::{check, UnresolvedArrowBody};

fn violations(dag: &v3_compiler::Dag) -> Vec<UnresolvedArrowBody> {
    check(dag)
}

#[test]
fn lens_silent_on_named_user_defined_fn() {
    let dag = compile_to_dag("fn foo(x: Int) -> Int = x", "user.v3").expect("compiles");
    let found = violations(&dag);
    assert!(
        found.is_empty(),
        "user fn with UserDefined body must not be flagged, got {} hit(s)",
        found.len()
    );
}

#[test]
fn lens_silent_on_anonymous_arrow_type_expression() {
    let dag =
        compile_to_dag("type Callback { handler: fn(Int) -> Int }", "user.v3").expect("compiles");
    let found = violations(&dag);
    assert!(
        found.is_empty(),
        "anonymous fn-type expressions must not be flagged, got {} hit(s)",
        found.len()
    );
}

#[test]
fn lens_silent_on_named_type_alias_to_arrow() {
    let dag = compile_to_dag("type Callback = fn(Int) -> Int", "user.v3").expect("compiles");
    let found = violations(&dag);
    assert!(
        found.is_empty(),
        "named type-alias arrow (NoBody) must not be flagged, got {} hit(s)",
        found.len()
    );
}
