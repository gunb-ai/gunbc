#!/usr/bin/env python3
"""Add the COLLISION RED the parent asked for (msg_78d4eb21) to the #14152 test file.

Two calls to the same zero-value-parameter generic function, bound to DIFFERENT concrete
Maps. `fn empty_map<K, V>() -> Map<K, V>` (dag/std/algebra.dag:1406; 1489 corpus calls, all
bare `empty_map()`) has no actuals, so the substitution is empty and the callee's declared
`Map(K, V)` was handed over as the call expression's settled type.

The repair seeds an unbound variable for each declared parameter the actuals did not solve.
Keying that variable on the BARE PARAMETER NAME would be producer 2's defect (bare_map_node
minting std's own declared names) reproduced at the call site: the two calls share one `K`,
and one call's instance can be written into the other's binding. So the fresh id carries the
call site. This block is the row that makes that claim checkable.

Two rows:
  * collision_two_zero_arg_generic_calls_both_type -- the parent's case verbatim: no hard
    diagnostic, no conflict, no cross-binding (a cross-binding shows up as the second
    declaration's settled instance refusing the first call's binding).
  * collision_pair_still_refuses_when_instantiated_twice -- refusal twin at the fold itself:
    two distinct SETTLED instances of the same head must still refuse, so the accept row
    cannot be satisfied by an implementation that admits anything involving a variable.

DISCRIMINATION IS MEASURED, NOT ASSUMED: run with --mutate name-keyed (the patcher
patch_producer_instantiate.py). If that mutation PASSES the first row, the collision is not
observable in this harness and that must be reported as a limit, not written off.

Usage: patch_collision_test.py REPO
"""
import sys

REPO = sys.argv[-1]
PATH = REPO + "/src/v1/tests/src/cross_coproduct_generic_locate_test.rs"

MARKER = "collision_two_zero_arg_generic_calls_both_type"

# Imports the new rows need; each is spelled exactly as an existing test file spells it
# (field_of_fractions_construction_test.rs), because that file is the precedent for
# compiling a source against the REAL std tree.
NEEDED_IMPORTS = [
    "use v1_compiler::cli_run;",
    "use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};",
    "use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};",
]

BLOCK = r'''
// ---------------------------------------------------------------------------
// PRODUCER REPAIR -- collision controls (parent msg_78d4eb21).
//
// A generic function with NO value parameters hands its declared result type over
// uninstantiated: with no actuals the substitution is empty and substitute_generics_apply
// short-circuits on it, so `empty_map()` was typed as the DECLARATION's own `Map(K, V)` and
// the fold then refused the settled instance that arrived later. The repair seeds an
// unbound variable per declared parameter the actuals did not solve. Naming that variable
// after the bare parameter would make two calls to the SAME generic function share it --
// producer 2's defect (bare_map_node minting std's own declared names) at the call site --
// so the id carries the call site. These rows are why.
// ---------------------------------------------------------------------------

const COLLISION_SOURCE: &str = r#"
module test.producer_collision

import std.types { Map }
import std.algebra { empty_map }

fn takes_int_map(m: Map<Int, Int>) -> Int { 0 }
fn takes_string_map(m: Map<String, Int>) -> Int { 0 }

// BOTH calls, ONE body. Two separate functions could never expose a shared variable: the
// cross-binding only exists where both call types meet inside a single unification, which is
// this body. Each call is constrained against a settled formal, so under a name-keyed
// unbound variable the first call fixes K,V := Int,Int and the second then refuses against
// String,Int -- a conflict that has nothing to do with the two maps' own types.
fn collide() -> Int {
  let a = empty_map()
  let b = empty_map()
  let x = takes_int_map(a)
  let y = takes_string_map(b)
  x + y
}
"#;

fn collision_resolved() -> Rc<ResolvedPipelineResult> {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources =
        resolve_imports_transitively_with_source_roots("test.dag", COLLISION_SOURCE, &roots);
    compile_to_resolved(Rc::new(sources.into()))
}

/// THE COLLISION ROW (the parent's case). Two calls to one zero-argument generic function,
/// bound to two different concrete Maps, must both type: no hard diagnostic, no conflict,
/// no cross-binding. Cross-binding would surface here as the second declaration's settled
/// instance refusing the binding the first call took -- a wrong answer is what the shared
/// variable produces, and the row fails on it either loudly or silently-quietly.
///
/// RED-BEFORE is expected: pre-repair both calls carry the callee's own Map(K, V).
#[test]
fn collision_two_zero_arg_generic_calls_both_type() {
    let resolved = collision_resolved();
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .filter(|d| cli_run::compile_clean_diagnostic_is_hard(d))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "two empty_map() calls bound to Map(Int, Int) and Map(String, Int) must both type: {:?}",
        msgs,
    );
}

/// REFUSAL TWIN, built at the fold itself so nothing upstream can satisfy it: two distinct
/// SETTLED instances of the same head must still refuse. Without this row the accept above
/// could be satisfied by an implementation that admits any pair merely because a variable
/// is involved somewhere.
#[test]
fn collision_pair_still_refuses_when_instantiated_twice() {
    let bound = fold_app("Map", &[fold_leaf("Int"), fold_leaf("Int")]);
    let actual = fold_app("Map", &[fold_leaf("String"), fold_leaf("Int")]);
    let acc = fold_step(bound, actual);
    assert!(
        !acc.diagnostics.is_empty(),
        "Map(Int, Int) then Map(String, Int) are two settled instances and must refuse: {:?}",
        acc.diagnostics
            .iter()
            .map(|d| format!("{:?}", d))
            .collect::<Vec<String>>()
    );
}
'''


def main():
    src = open(PATH, encoding="utf-8").read()
    if MARKER in src:
        sys.exit("FATAL: collision block already present")
    # fold helpers must already be declared above the append point.
    for needle in ("fn fold_step(", "fn fold_app(", "fn fold_leaf("):
        if needle not in src:
            sys.exit("FATAL: expected %r in the test file, absent -- do not patch blind" % needle)
    missing = [i for i in NEEDED_IMPORTS if i not in src]
    if missing:
        # Append the missing use-lines after the last existing use-line at the top.
        lines = src.split("\n")
        last = max(i for i, l in enumerate(lines) if l.startswith("use "))
        lines[last + 1 : last + 1] = missing
        src = "\n".join(lines)
    out = src.rstrip("\n") + "\n" + BLOCK
    open(PATH, "w", encoding="utf-8").write(out)
    print("appended 2 collision rows to %s (imports added: %d)" % (PATH, len(missing)))


main()
