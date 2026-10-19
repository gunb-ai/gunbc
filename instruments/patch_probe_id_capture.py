#!/usr/bin/env python3
"""Append the id-extraction PROBE rows to the collision test file (scratch only).

WHY A PRINTER AND NOT AN ASSERTION. Parent's ruling (b) needs a RED that fails under the name-keyed
mutant. But before asserting what sharing means, I have to know what the compiler actually minted --
asserting a guessed id shape would be exactly the guess the probe exists to replace, and a row that
asserts `id == "K"` versus `id == "K@file:1-2"` can be green for the wrong reason (both calls' ids
distinct for a reason unrelated to my repair). These rows PRINT; the permanent rows then assert the
semantics the print reveals.

They are probe-only by design and must not reach session/quiet-hawk-507 or the WIP ref: they print to
stdout and assert only structural facts (the walk found *something*), so they cannot rot into a lock
on a string format. The permanent fold-level control that parent asked for is a separate change.

usage: patch_probe_id_capture.py <repo> 
"""
import sys, pathlib

TARGET = "src/v1/tests/src/cross_coproduct_generic_locate_test.rs"

PROBE = r'''
// ---------------------------------------------------------------------------
// PROBE (scratch line only -- never the WIP ref, never the PR head): what variable
// ids did the COMPILER mint for the two empty_map() calls?
//
// The permanent control has to consume ids the compiler produced; hand-built ids test
// unify_generics' semantics, not the call-site minting, and would be invariant to the
// name-keyed mutant. So the ids come off the resolved tree. Printed, not asserted: the
// assertion belongs in the permanent row, once the print says what is true.
// ---------------------------------------------------------------------------

/// Every TypeVariable id reachable from a node, with the path taken to it. Depth-bounded
/// because the tree is the whole std closure once modules are linked, and this only needs
/// one function body.
fn collect_type_variable_ids(n: &Rc<Node>, depth: u32, out: &mut Vec<String>) {
    if depth > 12 || out.len() > 400 {
        return;
    }
    if let Some(inf) = &n.inferred {
        match &**inf {
            InferredNode::TypeVariable { id } => {
                out.push(format!("TV head={} id={}", n.name, id));
            }
            InferredNode::Resolved { node } => collect_type_variable_ids(node, depth + 1, out),
            _ => {}
        }
    }
    for c in n.children.iter() {
        collect_type_variable_ids(c, depth + 1, out);
    }
    if let Some(b) = &n.body {
        collect_type_variable_ids(b, depth + 1, out);
    }
    for p in n.params.iter() {
        collect_type_variable_ids(p, depth + 1, out);
    }
}

/// Find `fn collide()` in the resolved tree and print its variable ids.
fn probe_collide_ids(label: &str) -> Vec<String> {
    let resolved = collision_resolved();
    let mut found: Vec<String> = Vec::new();
    let mut visited = 0usize;
    let mut names: Vec<String> = Vec::new();
    if let Some(graph) = &resolved.graph {
        for m in graph.modules.iter() {
            for item in m.items.iter() {
                visited += 1;
                // name only: Node.ident is Option<Rc<String>> and comparing it to a &str is a
                // type error waiting to happen; the name is what the parser sets for a fn.
                if item.name == "collide" {
                    collect_type_variable_ids(item, 0, &mut found);
                }
                if names.len() < 40 {
                    names.push(item.name.clone());
                }
            }
        }
    }
    println!("PROBE {label} items_visited={visited} ids_found={}", found.len());
    // Printed so a zero is diagnosable rather than mysterious: if the name I searched for is not
    // in this list, the walk simply looked at the wrong item, and that is a different failure
    // from "the compiler minted nothing".
    println!("PROBE {label} ITEMS {names:?}");
    for l in &found {
        println!("PROBE {label} LINE {l}");
    }
    // Structural, not a guess about the id FORMAT: the body must be reachable and must carry
    // at least the two Map argument positions. If this is zero the walker missed the path and
    // every conclusion from the print would be meaningless -- so it fails loudly here.
    // The prints above land BEFORE this fires, so a zero still tells me what happened. The
    // assert exists so an empty print can never be read as "the compiler minted nothing": a
    // missing item is a broken walk, and conclusions about sharing need a walk that worked.
    assert!(
        !found.is_empty(),
        "PROBE {label}: the walk found no type-variable ids under fn collide() \
         (items visited: {visited}, items: {names:?}) -- the tree path is wrong, do not \
         interpret the print as the compiler having minted nothing"
    );
    found
}

#[test]
fn probe_prints_the_minted_ids_for_both_calls() {
    let ids = probe_collide_ids("collide");
    let distinct: std::collections::HashSet<&String> = ids.iter().collect();
    // Printed for the record either way; the count is a structural sanity check, not the
    // claim under test (which is id SHAPE, and that is read from the print).
    println!(
        "PROBE collide SUMMARY total={} distinct={}",
        ids.len(),
        distinct.len()
    );
    let site_tagged: Vec<&&String> = ids.iter().filter(|s| s.contains('@')).collect();
    println!(
        "PROBE collide SUMMARY site_tagged={} bare={}",
        site_tagged.len(),
        ids.len() - site_tagged.len()
    );
}

/// The fold-level capture parent asked for, run the SAME way at both trees: does
/// `unify_generics` itself distinguish a shared unbound variable from two distinct ones?
/// Hand-built nodes, so this costs no std compile; it answers a question about the FOLD,
/// and it is deliberately capture-mode -- the permanent row asserts whichever answer is
/// true, and if neither refuses then the fold cannot make the distinction and that is the
/// limit I report rather than a row that pretends otherwise.
#[test]
fn probe_fold_shared_vs_distinct_variable_identity() {
    // Shared: both argument positions name the SAME unbound variable K/V.
    let shared = fold_app("Map", &[fold_tv_leaf("K"), fold_tv_leaf("V")]);
    // Distinct: same shapes, ids tagged so no two leaves name the same variable.
    let distinct = fold_app(
        "Map",
        &[
            fold_tv_leaf("K@src/v2/std/text.dag:1-2"),
            fold_tv_leaf("V@src/v2/std/text.dag:3-4"),
        ],
    );
    let settled_int = fold_app("Map", &[fold_leaf("Int"), fold_leaf("Int")]);
    let settled_str = fold_app("Map", &[fold_leaf("String"), fold_leaf("Int")]);

    // Shape A: the binding is the unbound form, then two DIFFERENT settled instances arrive.
    let a_shared = fold_step_shared_then_two(shared.clone(), &settled_int, &settled_str);
    let a_distinct = fold_step_shared_then_two(distinct.clone(), &settled_int, &settled_str);
    println!(
        "PROBE fold shared_diag={} distinct_diag={} shared_msgs={:?} distinct_msgs={:?}",
        a_shared.diagnostics.len(),
        a_distinct.diagnostics.len(),
        a_shared
            .diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>(),
        a_distinct
            .diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
    println!(
        "PROBE fold subst_shared_has_K={:?} subst_distinct_has_K={:?}",
        a_shared.subst.keys().collect::<Vec<_>>(),
        a_distinct.subst.keys().collect::<Vec<_>>()
    );
}

/// Two settled actuals against one unbound binding, in the fold's own calling convention.
/// Mirrors fold_step's setup so the comparison is with the existing rows, not a new one.
fn fold_step_shared_then_two(
    unbound: Rc<Node>,
    first: &Rc<Node>,
    second: &Rc<Node>,
) -> UnifyGenericsAcc {
    let si = Rc::new(im::HashMap::new());
    let mut acc: im::HashMap<String, Rc<Node>> = im::HashMap::new();
    acc.insert("T".to_string(), unbound);
    let step1 = unify_generics(
        fold_leaf("T"),
        first.clone(),
        Rc::new(im::Vector::from(vec!["T".to_string()])),
        si.clone(),
        Rc::new(acc.clone()),
        Rc::new(TextJudgment::TextNotAsked {
            reason: TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes,
        }),
        "locate.probe".to_string(),
    );
    unify_generics(
        fold_leaf("T"),
        second.clone(),
        Rc::new(im::Vector::from(vec!["T".to_string()])),
        si,
        step1.subst.clone(),
        Rc::new(TextJudgment::TextNotAsked {
            reason: TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes,
        }),
        "locate.probe".to_string(),
    )
}
'''


def main():
    repo = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
    p = repo / TARGET
    if not p.is_file():
        print(f"FATAL: {p} missing")
        return 1
    s = p.read_text()
    if "fn probe_prints_the_minted_ids_for_both_calls" in s:
        print("already applied")
        return 0
    # Anchor: the end of the file. The rows are additive and touch nothing existing.
    p.write_text(s.rstrip("\n") + "\n" + PROBE.lstrip("\n"))
    print(f"appended probe rows to {TARGET}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
