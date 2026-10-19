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

    // PRINT THE INVENTORY BEFORE DECIDING, so a PASS is auditable. Without this line a passing run
    // and a passing-but-vacuous run look identical in the log: the assertions below can only tell me
    // the ids were not the forbidden ones, not what they actually were. Under --nocapture this is
    // what a reviewer reads, and it is what the runner echoes as ROW_PRINTS.
    let tags: std::collections::HashSet<&str> = distinct
        .iter()
        .map(|i| i.split_once('@').map(|(_, t)| t).unwrap_or("<UNTAGGED>"))
        .collect();
    println!(
        "MINT-INVENTORY positions={} distinct={} site_tags={} bare={} sample={:?}",
        ids.len(),
        distinct.len(),
        tags.len(),
        distinct.iter().filter(|i| !i.contains('@')).count(),
        distinct.iter().take(6).collect::<Vec<_>>()
    );

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
    // `tags` is the same set the inventory above printed, bound once, so the number a reviewer
    // reads and the number the assertion uses cannot drift apart.
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

// ---------------------------------------------------------------------------
// PERMANENT: the narrowing pin. `uninstantiated_call_generics` mints ONLY for a call that carries
// no argument nodes. This row is the reason that guard cannot be quietly widened: it asks for a
// generic call that DOES pass an argument and DOES leave its declared parameter unbound, and
// requires main's behaviour to survive untouched.
//
// The callee is `fn time_measure<S>(count: Nat) -> Measure<Time, S, Nat>`
// (dag/std/measure.dag:284). `S` appears only in the return type, so the argument cannot bind it:
// `unify_generics` records no binding, `map_has(call_subst, "S")` is false, and a mint would fire
// under the pre-narrowing guard. Minting there is not harmless -- it replaces the CALLER's own `S`
// with a variable no actual mentions, and because the emitter's clone-bound pre-pass keys on the
// parameter NAME (src/v1/05_emit_rust.dag, clone_param), the emitted signature silently loses its
// `S: Clone` bound. That is exactly how PR-1's head first went red on `v2-bootstrap`:
// error[E0277] `S: Clone` at stage0/src/std_realization_measurement.rs:341:72 in the self-host
// product, with main green on the same gate.
// ---------------------------------------------------------------------------

const NARROWING_SOURCE: &str = r#"
module test.narrowing_pin

import std.measure { time_measure }

// ONE call, WITH an argument, whose declared parameter `S` the argument cannot reach. Written to
// match dag/test/claim/realization_scale_agnostic.dag:4, an in-tree call of the same function, so
// the row cannot go red for a reason unrelated to the mint.
fn pinned() -> Bool {
  let m = time_measure(count: 10)
  m.count == 0
}
"#;

fn pinned_minted_state() -> (Vec<String>, usize) {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources =
        resolve_imports_transitively_with_source_roots("test.dag", NARROWING_SOURCE, &roots);
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let mut ids: Vec<String> = Vec::new();
    let mut saw_fn = false;
    let mut call_nodes = 0usize;
    if let Some(graph) = &resolved.graph {
        for m in graph.modules.iter() {
            for item in m.items.iter() {
                if item.name == "pinned" {
                    saw_fn = true;
                    type_variable_ids_under(item, 0, &mut ids);
                    call_nodes = count_named_nodes(item, "time_measure", 0);
                }
            }
        }
    }
    // Both guards are fatal: a missing item or a call that stopped being a call would let the
    // assertions below pass while the row measured nothing at all.
    assert!(
        saw_fn,
        "fn pinned() is not in the resolved tree -- the row would pass by looking at nothing"
    );
    assert!(
        call_nodes >= 1,
        "no `time_measure` call node under fn pinned() -- the call the narrowing pins is not in \
         the tree, so this row would prove nothing about it (found {call_nodes})"
    );
    (ids, call_nodes)
}

/// Count nodes whose name matches, depth-bounded like `type_variable_ids_under`.
fn count_named_nodes(n: &Rc<Node>, name: &str, depth: u32) -> usize {
    if depth > 12 {
        return 0;
    }
    // Suffix, not equality: a resolved call node may carry the qualified name
    // ("std.measure.time_measure"), and matching only the bare name would trip the fatal guard in
    // the caller and fail the row for a naming accident rather than for the claim.
    let mut hits = usize::from(n.name.ends_with(name));
    for c in n.children.iter() {
        hits += count_named_nodes(c, name, depth + 1);
    }
    if let Some(b) = &n.body {
        hits += count_named_nodes(b, name, depth + 1);
    }
    for p in n.params.iter() {
        hits += count_named_nodes(p, name, depth + 1);
    }
    hits
}

#[test]
fn arguments_present_do_not_trigger_the_call_site_mint() {
    let (ids, call_nodes) = pinned_minted_state();
    let distinct: std::collections::HashSet<&String> = ids.iter().collect();
    let minted: Vec<&&String> = distinct.iter().filter(|i| i.contains('@')).collect();

    println!(
        "NARROWING-INVENTORY positions={} distinct={} minted={} calls={} sample={:?}",
        ids.len(),
        distinct.len(),
        minted.len(),
        call_nodes,
        distinct.iter().take(6).collect::<Vec<_>>()
    );

    // The whole claim: nothing call-site-tagged may appear, because this call HAS an argument.
    assert!(
        minted.is_empty(),
        "the call-site mint fired at a call that passes an argument: {minted:?} -- minting here \
         fabricates a binding unify_generics did not make, replaces the caller's own type \
         variable, and silently drops the emitted `S: Clone` bound (the pre-narrowing guard \
         produced ids like \"S@<file>:<start>-<end>\" at exactly this shape)"
    );
    // And the parameter must still be present as main leaves it, so the row cannot be satisfied by
    // the call simply having lost its type.
    assert!(
        distinct.iter().any(|i| **i == "S"),
        "no type variable `S` reachable under fn pinned(): {distinct:?} -- expected main's \
         behaviour to survive untouched (the callee's own declared parameter), so the narrowing \
         is not merely suppressing minting but changing what the call types to"
    );
}
