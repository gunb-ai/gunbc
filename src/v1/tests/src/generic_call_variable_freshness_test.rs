// PRODUCER-REPAIR CONTROLS.
//
// Scope note, because this file is deliberately a subset of the lane's controls. The fold-level
// controls that exercise `UnifyGenericsAcc` belong to the follow-up PR and cannot appear here: that
// accumulator changed `unify_generics`'s signature from five arguments to six, so a row calling it
// directly is a compile error against a base with no accumulator. What stays is what measures the
// PRODUCER -- the ids the compiler mints for a generic call's unbound parameters -- which is the
// half that can be proven without any conflict reporting.
use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};
use std::rc::Rc;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_std_core::{InferredNode, Node};

const COLLISION_SOURCE: &str = r#"
module test.producer_collision

import std.types { Map }
import std.algebra { empty_map }

fn takes_int_map(m: Map<Int, Int>) -> Int { 0 }
fn takes_string_map(m: Map<String, Int>) -> Int { 0 }

// BOTH calls in ONE body, the first constrained before the second is introduced. Two
// separate top-level functions could never expose a shared unbound variable: whatever the
// two calls share, each is checked only against its own formal, so no comparison between
// them ever happens. Inside one body, the first call's constraints land on the shared
// variable before the second call is compared against a DIFFERENT settled instance -- which
// is where a name-keyed variable turns into a wrong answer.
fn collide() -> Int {
  let a = empty_map()
  let n = takes_int_map(a)
  let b = empty_map()
  let m = takes_string_map(b)
  n + m
}
"#;

fn collision_resolved() -> Rc<ResolvedPipelineResult> {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources =
        resolve_imports_transitively_with_source_roots("test.dag", COLLISION_SOURCE, &roots);
    compile_to_resolved(Rc::new(sources.into()))
}

// ---------------------------------------------------------------------------
// PERMANENT: call-site variable freshness. The property is "two unrelated calls to the
// same generic function must not share a variable" -- a silent wrong binding is the
// failure mode, so the control asserts on the ids the COMPILER minted, not on a proxy.
// ---------------------------------------------------------------------------

/// The declared parameter names of `empty_map<K, V>`, i.e. exactly what a name-keyed mint would
/// leave on the wire. Kept as data, not inlined in the assertion, so a future rename in std.algebra
/// shows up here rather than silently weakening the row.
const EMPTY_MAP_DECLARED_PARAMS: &[&str] = &["K", "V"];

/// Type-variable ids reachable from a node. Depth-bounded: the linked tree is the whole std
/// closure and this needs one function body, not the universe.
fn type_variable_ids_under(n: &Rc<Node>, depth: u32, out: &mut Vec<String>) {
    if depth > 12 || out.len() > 400 {
        return;
    }
    if let Some(inf) = &n.inferred {
        match &**inf {
            InferredNode::TypeVariable { id } => out.push(id.clone()),
            InferredNode::Resolved { node } => type_variable_ids_under(node, depth + 1, out),
            _ => {}
        }
    }
    for c in n.children.iter() {
        type_variable_ids_under(c, depth + 1, out);
    }
    if let Some(b) = &n.body {
        type_variable_ids_under(b, depth + 1, out);
    }
    for p in n.params.iter() {
        type_variable_ids_under(p, depth + 1, out);
    }
}

/// Ids on `fn collide()`, which is two `empty_map()` calls in ONE body (separate top-level
/// functions can never expose sharing: each call is only ever checked against its own formal).
fn collide_minted_ids() -> Vec<String> {
    let resolved = collision_resolved();
    let mut ids: Vec<String> = Vec::new();
    let mut saw_collide = false;
    if let Some(graph) = &resolved.graph {
        for m in graph.modules.iter() {
            for item in m.items.iter() {
                if item.name == "collide" {
                    saw_collide = true;
                    type_variable_ids_under(item, 0, &mut ids);
                }
            }
        }
    }
    // A missing item would make every assertion below vacuously true, so it is fatal here rather
    // than left to the count checks.
    assert!(
        saw_collide,
        "fn collide() is not in the resolved tree -- the row would pass by looking at nothing"
    );
    assert!(
        !ids.is_empty(),
        "no type-variable ids under fn collide() -- the walk found nothing, which is not the same \
         as freshness (measured on the repaired tree: 24 ids, 4 distinct)"
    );
    ids
}

#[test]
fn minted_variables_are_fresh_per_call_site() {
    let ids = collide_minted_ids();
    let distinct: std::collections::HashSet<&String> = ids.iter().collect();

    // (a) No variable may be left named by the callee's declared parameter name. That is precisely
    // the name-keyed mint: it hands two calls the same `K`, so a later call's settled Map(String,Int)
    // meets an earlier call's constraint on a variable they both own.
    for bad in EMPTY_MAP_DECLARED_PARAMS {
        assert!(
            !distinct.iter().any(|i| i == bad),
            "a minted variable is named {bad:?} -- the callee's own parameter name, so the two \
             empty_map() calls in fn collide() share it (measured at the name-keyed mutant: \
             distinct={distinct:?})"
        );
    }
    // (b) The two calls must carry DIFFERENT variables. The fixture is exactly two calls, so the
    // site tags -- the part after '@', which is the call's own span -- must number at least two.
    let tags: std::collections::HashSet<&str> = distinct
        .iter()
        .map(|i| i.split_once('@').map(|(_, t)| t).unwrap_or("<UNTAGGED>"))
        .collect();
    assert!(
        tags.len() >= 2,
        "the two empty_map() calls are not distinguishable by call site: tags={tags:?} \
         (measured at the repaired head: 4 distinct ids, 2 site tags; at the mutant: 2 ids, \
         0 tags)"
    );
    assert!(
        !tags.contains("<UNTAGGED>"),
        "some minted variable carries no call-site tag at all: {distinct:?}"
    );
}
