// CALL-SITE MINT BOUNDARY — a permanent forward guard, stated honestly about what it does today.
//
// WHAT THIS IS. A generic call whose declared parameter the actuals cannot reach is the shape a
// call-site mint would have to handle. The emitter cannot currently carry a minted variable at all
// (three sites key variable identity on the AUTHORED NAME: `tn == var_id` in src/v1/05_emit.dag, the
// `BRIDGE: fold empty_map accumulator type unresolved` in src/v1/05_emit_rust.dag, and `clone_param`
// there), so the mint is NOT part of this change — see
// gunbc.recurring_failure_mode.silent_first_wins_rebind_of_a_generic_type_variable_in_unify_generics,
// which records those sites and the two unify_generics consequences. Minting is a separate change,
// blocked on the emitter keying rendering and clone bounds on a stable variable identity.
//
// WHAT THIS ROW GUARDS. Whatever that change turns out to be, it must NOT mint at a call that passes
// actuals: absence from `call_subst` there means `unify_generics` failed to bind the parameter, and
// minting would FABRICATE a binding for something the actuals were meant to settle — a silent wrong
// binding, the class the row names. Today the row passes because nothing mints; that is not a
// weakness to hide, it is the baseline it pins. It earns its keep the moment the mint lands: fired at
// this shape, it goes red.
//
// Why this shape specifically. `fn time_measure<S>(count: Nat) -> Measure<Time, S, Nat>`
// (dag/std/measure.dag:284) passes ONE argument and its `S` appears only in the return type, so the
// argument cannot bind it: `map_has(call_subst, "S")` is false. That is precisely the ambiguous case,
// and it is not hypothetical — a mint that fired here cost the self-host product its `S: Clone` bound
// (the emitted signature lost the bound because the clone-bound pre-pass keys on the parameter NAME)
// and it failed `v2-bootstrap` with error[E0277] `S: Clone` at
// stage0/src/std_realization_measurement.rs:341:72, with main green on the same gate.
use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};
use std::rc::Rc;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_std_core::Node;

const PINNED_SOURCE: &str = r#"
module test.mint_boundary

import std.measure { time_measure }

// ONE call, WITH an argument, whose declared parameter `S` the argument cannot reach. Written to
// match dag/test/claim/realization_scale_agnostic.dag:4, an in-tree call of the same function, so the
// row cannot go red for a reason unrelated to the call's typing.
fn pinned() -> Bool {
  let m = time_measure(count: 10)
  m.count == 0
}
"#;

fn pinned_resolved() -> Rc<ResolvedPipelineResult> {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources = resolve_imports_transitively_with_source_roots("test.dag", PINNED_SOURCE, &roots);
    compile_to_resolved(Rc::new(sources.into()))
}

/// Type-variable ids reachable from a node. Depth-bounded: the linked tree is the whole std closure
/// and this needs one function body, not the universe.
fn type_variable_ids_under(n: &Rc<Node>, depth: u32, out: &mut Vec<String>) {
    if depth > 12 || out.len() > 400 {
        return;
    }
    if let Some(inferred) = &n.inferred {
        if let v1_compiler::v1_std_core::InferredNode::TypeVariable { id } = &**inferred {
            out.push(id.clone());
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

/// Count nodes whose name matches, depth-bounded like `type_variable_ids_under`. Matched by suffix:
/// a resolved call node may carry the qualified name ("std.measure.time_measure"), and matching only
/// the bare name would trip the fatal guard below for a naming accident rather than for the claim.
fn count_named_nodes(n: &Rc<Node>, name: &str, depth: u32) -> usize {
    if depth > 12 {
        return 0;
    }
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

/// Reach the pinned call. Both guards are FATAL: a missing item, or a call that stopped being a call,
/// would otherwise let the assertions below pass while the row measured nothing at all.
fn pinned_state() -> (Vec<String>, usize) {
    let resolved = pinned_resolved();
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
    assert!(
        saw_fn,
        "fn pinned() is not in the resolved tree -- the row would pass by looking at nothing"
    );
    assert!(
        call_nodes >= 1,
        "no `time_measure` call node under fn pinned() (found {call_nodes}) -- the call this row \
         pins is not in the tree, so it would prove nothing about it"
    );
    (ids, call_nodes)
}

#[test]
fn a_call_that_passes_actuals_gets_no_call_site_mint() {
    let (ids, call_nodes) = pinned_state();
    let distinct: std::collections::HashSet<&String> = ids.iter().collect();
    // A minted variable is tagged with the call site. Any of '@', ':' or '-' in an id is a site tag:
    // the shapes tried were "K@<file>:<start>-<end>" (span-keyed) and bare "K" (name-keyed).
    let is_site_tagged = |id: &&String| id.contains('@') || id.contains(':') || id.contains('-');
    let minted: Vec<&&String> = distinct.iter().filter(is_site_tagged).collect();

    println!(
        "MINT-BOUNDARY-INVENTORY positions={} distinct={} minted={} calls={} sample={:?}",
        ids.len(),
        distinct.len(),
        minted.len(),
        call_nodes,
        distinct.iter().take(6).collect::<Vec<_>>()
    );

    assert!(
        minted.is_empty(),
        "a call-site-tagged type variable appeared at a call that PASSES AN ACTUAL: {minted:?}. \
         Absence from call_subst at a call with arguments means unify_generics failed to bind the \
         parameter; minting there fabricates a binding and replaces a variable the caller named."
    );
    // The parameter must still be present as main leaves it, so the row cannot be satisfied by the
    // call having lost its type: green must mean "main's typing, untouched", not "nothing to see".
    assert!(
        distinct.iter().any(|i| i.as_str() == "S"),
        "no type variable `S` reachable under fn pinned(): {distinct:?} -- expected main's behaviour \
         to survive untouched (the callee's own declared parameter)"
    );
}
