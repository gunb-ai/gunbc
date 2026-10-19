#!/usr/bin/env python3
"""Land the permanent freshness rows on the #14152 test file.

WHY THESE THREE, and why not the shape originally asked for. The request was a fold-level control
that calls `unify_generics` with two `Map(TypeVariable)` actuals -- shared ids (must refuse) and
distinct per-site ids (must accept) -- such that the name-keyed mutant fails it. Measured at
9c949cabe5 / c2d582ee7b (PREDICTIONS_14152_producer_repair.md AMENDMENT 8), the fold's acc is keyed by
the FORMAL name and its leaves' variable ids are never compared to each other, so BOTH legs refuse with
the SAME diagnostic. The "distinct must accept" leg is therefore false as specified: building it would
produce a row that is red on a correct tree. And because the fold cannot see leaf identity, no
hand-built fold-level row can fail under the mutant -- it is invariant by construction.

So the RED goes where the property actually lives, on the ids the COMPILER mints:

  1. minted_variables_are_fresh_per_call_site  -- the mutant FAILS this. Measured: the shipped repair
     mints K@test.dag:735-744 / V@…735-744 / K@…784-793 / V@…784-793 (4 distinct, all site-tagged);
     the mutant mints bare K / V (2 distinct, none tagged) and the two calls share them.
  2. fold_refuses_a_settled_rebind_of_a_declaration_form_binding -- the correct half of the requested
     pair: a declaration-form binding followed by two different settled instances must refuse, and the
     diagnostic must name the binding it is refusing against.
  3. fold_cannot_distinguish_shared_from_distinct_leaf_identity -- locks the LIMIT in the suite rather
     than only in a predictions file: shared and distinct leaf ids produce byte-identical diagnostics.
     If someone later adds a leaf-identity comparison to the fold, this row goes red and points at the
     measurement that said it was absent.

usage: patch_permanent_freshness_rows.py <repo>
"""
import sys, pathlib

TARGET = "src/v1/tests/src/cross_coproduct_generic_locate_test.rs"

ROWS = r'''
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

/// The correct leg of the requested fold pair: a binding that is the callee's own generic
/// declaration form, then two DIFFERENT settled instances, must refuse -- and say what it
/// refused against. Measured: 1 diagnostic, naming the binding, at both trees.
#[test]
fn fold_refuses_a_settled_rebind_of_a_declaration_form_binding() {
    let acc = fold_step_shared_then_two(
        fold_app("Map", &[fold_tv_leaf("K"), fold_tv_leaf("V")]),
        &fold_app("Map", &[fold_leaf("Int"), fold_leaf("Int")]),
        &fold_app("Map", &[fold_leaf("String"), fold_leaf("Int")]),
    );
    let msgs: Vec<String> = acc
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    assert_eq!(
        msgs.len(),
        1,
        "a settled rebind of a declaration-form binding must refuse exactly once, got {msgs:?}"
    );
    assert!(
        msgs[0].contains("type variable T bound as"),
        "the refusal must name the binding it refused against, got {:?}",
        msgs[0]
    );
    assert!(
        msgs[0].contains("Map(Primitive(String)"),
        "the refusal must name the actual that conflicted, got {:?}",
        msgs[0]
    );
}

/// LOCKS A LIMIT, does not assert a behaviour I wish were true. The fold's accumulator is keyed by
/// the FORMAL name and its leaves' variable ids are never compared, so shared and distinct leaf
/// identity produce identical refusals. Measured at both c2d582ee7b and 9c949cabe5: 1 diagnostic
/// each, same message, subst keys ["T"]. If a future change gives the fold a leaf-identity path,
/// this row goes red and points back at that measurement instead of at a vague expectation.
#[test]
fn fold_cannot_distinguish_shared_from_distinct_leaf_identity() {
    let shared = fold_app("Map", &[fold_tv_leaf("K"), fold_tv_leaf("V")]);
    let distinct = fold_app(
        "Map",
        &[
            fold_tv_leaf("K@src/v2/std/text.dag:1-2"),
            fold_tv_leaf("V@src/v2/std/text.dag:3-4"),
        ],
    );
    let int_map = fold_app("Map", &[fold_leaf("Int"), fold_leaf("Int")]);
    let str_map = fold_app("Map", &[fold_leaf("String"), fold_leaf("Int")]);

    let msgs_of = |acc: Rc<UnifyGenericsAcc>| -> Vec<String> {
        acc.diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect()
    };
    let a = msgs_of(fold_step_shared_then_two(shared, &int_map, &str_map));
    let b = msgs_of(fold_step_shared_then_two(distinct, &int_map, &str_map));
    assert_eq!(
        a, b,
        "the fold now distinguishes shared from distinct leaf variable identity -- that is a real \
         behaviour change and this row is the record that it did not exist when the freshness work \
         landed (both legs measured 1 identical diagnostic)"
    );
    assert_eq!(
        a.len(),
        1,
        "measured: exactly one diagnostic; a change here means the fold's rebind rule moved"
    );
}

/// Two settled actuals against one declaration-form binding, in the fold's own calling convention.
/// Mirrors fold_step so this is a comparison with the existing rows, not a new harness.
fn fold_step_shared_then_two(
    unbound: Rc<Node>,
    first: &Rc<Node>,
    second: &Rc<Node>,
) -> Rc<UnifyGenericsAcc> {
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
    if "fn minted_variables_are_fresh_per_call_site" in s:
        print("already applied")
        return 0
    if "fn fold_step_shared_then_two" in s and "fn minted_variables_are_fresh_per_call_site" not in s:
        # The scratch probe rows define a helper with this name; applying the permanent rows on top
        # of them would emit two definitions of the same fn and the crate would not compile.
        print("FATAL: the probe rows are already applied here -- permanent and probe patches are "
              "mutually exclusive (duplicate fold_step_shared_then_two)")
        return 1
    if "fn collision_resolved" not in s:
        # The rows read the resolved tree of the collision fixture; without that helper they would
        # re-derive it differently and stop measuring the same program the collision row measures.
        print("FATAL: collision_resolved() missing -- these rows must share that fixture")
        return 1
    p.write_text(s.rstrip("\n") + "\n" + ROWS.lstrip("\n"))
    print(f"appended permanent rows to {TARGET}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
