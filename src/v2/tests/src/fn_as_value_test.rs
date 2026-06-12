//! Regression: module-level fn items resolve as first-class callable values (gap-a).

use std::fs;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use v2_compiler::cli_run;
use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::resolve_imports_transitively;

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

#[test]
fn fn_item_bound_and_called_via_first_class_reference() {
    let src = r#"module test.fn_as_value
fn add(a: Int, b: Int) -> Int { a + b }
fn use_via_binding() -> Int {
  let f = add
  f(a: 2, b: 3)
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v2_interpreter::run(graph, resolved.source_indices.clone(), "use_via_binding") {
        Ok(Value::Int(5)) => {}
        other => panic!("expected Int(5), got {other:?}"),
    }
}

#[test]
fn scoped_entry_resolves_import_closure_not_entire_v4_tree() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "gunbc-scoped-entry-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).expect("temp dir");

    let dep = "module test.scoped.dep\nfn dep_fn() -> Int { 1 }\n";
    let entry = "module test.scoped.entry\nimport test.scoped.dep { dep_fn }\nfn main() -> Int { dep_fn() }\n";
    let noise = "module test.scoped.noise\nfn noise_fn() -> Int { 0 }\n";
    fs::write(dir.join("dep.dag"), dep).expect("write dep");
    fs::write(dir.join("entry.dag"), entry).expect("write entry");
    fs::write(dir.join("noise.dag"), noise).expect("write noise");

    let roots = vec![dir.to_string_lossy().into_owned()];
    let entry_path = dir.join("entry.dag");
    let scoped = cli_run::load_sources_for_entry(&roots, entry_path.to_str().unwrap())
        .expect("load scoped closure");
    assert_eq!(
        scoped.len(),
        2,
        "expected entry + transitive import only, got paths: {:?}",
        scoped.iter().map(|s| s.path.as_str()).collect::<Vec<_>>()
    );

    let _ = fs::remove_dir_all(&dir);
}

// ── Generic value-arg instantiation (the emit(add) unblock) ──────────────
// A generic HOF binds its type parameter from a concrete value arg and
// substitutes it into a later lambda's param type, so the lambda body sees the
// CONCRETE type (field access resolves + evaluates). Before this, the lambda
// param stayed generic `T` and a field access cascaded to an error type.

#[test]
fn generic_call_instantiates_lambda_param_and_evaluates() {
    let src = r#"module test.gi
type Rec { v: Int }
fn apply_rec<T>(x: T, g: fn(T) -> Int) -> Int { g(x) }
fn use_it() -> Int { apply_rec(x: Rec { v: 7 }, g: fn(r) { r.v }) }
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "use_it") {
        Ok(Value::Int(7)) => {}
        other => panic!(
            "expected Int(7) (T bound to Rec from x, lambda r: Rec, r.v read), got {other:?}"
        ),
    }
}

#[test]
fn generic_instantiation_field_checks_concrete_type_red() {
    // Red-when-wrong: with T bound to Rec, the lambda param is concrete, so a
    // non-existent field must NOT silently succeed. If instantiation left the
    // param generic-and-permissive this would pass — it must not.
    let src = r#"module test.gi2
type Rec { v: Int }
fn apply_rec<T>(x: T, g: fn(T) -> Int) -> Int { g(x) }
fn use_bad() -> Int { apply_rec(x: Rec { v: 7 }, g: fn(r) { r.nope }) }
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    let has_diag = resolved
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .any(|m| !m.starts_with("complexity: "));
    let evaluates_ok = match resolved.graph.as_ref() {
        Some(g) => matches!(
            v2_interpreter::run(g, resolved.source_indices.clone(), "use_bad"),
            Ok(Value::Int(_))
        ),
        None => false,
    };
    assert!(
        has_diag || !evaluates_ok,
        "r.nope on instantiated Rec (T=Rec) must fail closed (diagnostic or eval error), not silently succeed"
    );
}

// ── Generic FreeMonoid fold callback instantiation (R2 §4a) ─────────────
// fold_list/fold_list_right cons|snoc callbacks must bind T from xs: FreeMonoid<T>
// so the second lambda param is concrete (field access / arithmetic type-checks).

#[test]
fn fold_list_generic_cons_callback_binds_element_type() {
    let src = r#"module test.fold_gi
type Rec { v: Int }
type IntList = Empty | Cons { head: Int, tail: IntList }
fn fold_list<T, A>(xs: FreeMonoid<T>, empty: A, cons: fn(A, T) -> A) -> A {
  match xs {
    Empty => empty
    Cons { head: h, tail: t } => fold_list(xs: t, empty: cons(empty: empty, h: h), cons: cons)
  }
}
fn sum_ints() -> Int {
  fold_list(xs: [1, 2, 3], empty: 0, cons: fn(acc, h) { acc + h })
}
fn field_access() -> Int {
  let xs = [Rec { v: 1 }, Rec { v: 2 }]
  fold_list(xs: xs, empty: 0, cons: fn(acc, r) { acc + r.v })
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "sum_ints") {
        Ok(Value::Int(6)) => {}
        other => panic!("expected Int(6) from fold_list sum, got {other:?}"),
    }
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "field_access") {
        Ok(Value::Int(3)) => {}
        other => panic!(
            "expected Int(3) (T=Rec from xs, cons r: Rec, r.v field access), got {other:?}"
        ),
    }
}

#[test]
fn fold_list_right_generic_snoc_callback_binds_element_type() {
    let src = r#"module test.fold_right_gi
type Rec { v: Int }
fn fold_list_right<T, A>(xs: FreeMonoid<T>, empty: A, snoc: fn(A, T) -> A) -> A {
  match xs {
    Empty => empty
    Cons { head: h, tail: t } => snoc(fold_list_right(xs: t, empty: empty, snoc: snoc), h)
  }
}
fn field_access() -> Int {
  let xs = [Rec { v: 4 }, Rec { v: 5 }]
  fold_list_right(xs: xs, empty: 0, snoc: fn(acc, r) { acc + r.v })
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "field_access") {
        Ok(Value::Int(9)) => {}
        other => panic!(
            "expected Int(9) (T=Rec from xs, snoc r: Rec, r.v field access), got {other:?}"
        ),
    }
}
