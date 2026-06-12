//! v2 R2 generic instantiation — NodeFold/NodeFoldTopDown algebra fn fields bind A,R
//! from the algebra type at fold_node_topdown/fold_node call sites (dep-graph §4a item b).

use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively, resolve_imports_transitively_with_source_roots, workspace_root};

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {msgs:?} (graph present: {})",
        result.graph.is_some()
    );
}

#[test]
fn nodefold_topdown_inline_algebra_binds_accumulator_fields() {
    // Repro for #4731 gap class: inline NodeFoldTopDown { init/step/child_context }
    // lambdas must see concrete A=R=MyFold so fold.origin/state/at_conj field access type-checks.
    let src = r#"module test.nodefold_gi
type MyFold {
  origin: Int
  state: Int
  at_conj: Bool
}
type Empty = Empty
type Cons<T> = Cons { head: T, tail: FreeMonoid<T> }
fn my_leaf() -> Node {
  Node { kind: TypeNode { connective: Atom { identity: ^leaf } }, children: [], occurrence_id: SyntheticOccurrence }
}
fn inline_topdown_algebra() -> NodeFoldTopDown<MyFold, MyFold> {
  NodeFoldTopDown {
    init: fn(n, fold) {
      MyFold { origin: fold.origin, state: fold.state + 1, at_conj: true }
    },
    child_context: fn(n, fold, e) { fold },
    step: fn(acc, e, child_r) {
      if acc.at_conj { child_r } else { acc }
    }
  }
}
fn use_inline() -> Int {
  match fold_node_topdown(
    n: my_leaf(),
    ctx: MyFold { origin: 7, state: 0, at_conj: false },
    algebra: inline_topdown_algebra()
  ) {
    MyFold { origin: o, state: s, at_conj: _ } => o + s
  }
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "use_inline") {
        Ok(Value::Int(8)) => {}
        other => panic!(
            "expected Int(8) (origin=7, state incremented to 1 in init), got {other:?}"
        ),
    }
}

#[test]
fn nodefold_inline_algebra_binds_result_type_fields() {
    let src = r#"module test.nodefold_r_gi
type MyAcc {
  total: Int
}
type Empty = Empty
type Cons<T> = Cons { head: T, tail: FreeMonoid<T> }
fn count_leaf() -> Node {
  Node { kind: TypeNode { connective: Atom { identity: ^leaf } }, children: [], occurrence_id: SyntheticOccurrence }
}
fn one_child() -> Node {
  Node {
    kind: TypeNode { connective: Conj },
    children: [Edge { label: Named { name: ^child }, target: count_leaf() }],
    occurrence_id: SyntheticOccurrence
  }
}
fn inline_nodefold_algebra() -> NodeFold<MyAcc> {
  NodeFold {
    init: fn(n) { MyAcc { total: 0 } },
    step: fn(acc, e, child) {
      MyAcc { total: acc.total + child.total + 1 }
    }
  }
}
fn use_nodefold() -> Int {
  fold_node(n: one_child(), algebra: inline_nodefold_algebra()).total
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "use_nodefold") {
        Ok(Value::Int(1)) => {}
        other => panic!("expected Int(1) from NodeFold inline algebra, got {other:?}"),
    }
}

#[test]
fn chained_generic_field_access_resolves_without_stepped_accessors() {
    // Repro for dep-graph §4a item (c): chained projection run.cache.key.claim without
    // stepped accessor fns. 05_eval.dag comment claims this fails — verify.
    let src = r#"module test.chain_gi
type Inner<S> { value: S }
type Middle<S> { inner: Inner<S> }
type Outer<S> { middle: Middle<S> }
type Rec { v: Int }
fn chained_field<S>(o: Outer<S>) -> S {
  o.middle.inner.value
}
fn use_chained() -> Int {
  chained_field(o: Outer { middle: Middle { inner: Inner { value: Rec { v: 9 } } } }).v
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "use_chained") {
        Ok(Value::Int(9)) => {}
        other => panic!(
            "expected Int(9) from chained generic field access, got {other:?}"
        ),
    }
}

#[test]
fn v4_fold_node_topdown_mvp_cert_compiles() {
    const ENTRY: &str = "src/v4/test/claim/manual/fold_node_topdown_mvp.dag";
    let entry_content = std::fs::read_to_string(workspace_root().join(ENTRY))
        .unwrap_or_else(|e| panic!("read {ENTRY}: {e}"));
    let sources = resolve_imports_transitively_with_source_roots(
        ENTRY,
        &entry_content,
        &[workspace_root().join("src/v4")],
    );
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
}
