#!/usr/bin/env python3
"""ACCEPTANCE ITEM 5 (msg_4199351d) -- DELETION TEST.

typed_expr_is_stale / recheck_expr / infer_planned_lambda_recheck exist to avoid
re-inferring a lambda argument whose prior typed form is still accurate. The revised bar
says: keep them ONLY if the right-bound series proves them necessary AND their depth curve
is bounded; if plain full lambda re-inference after the corrected still_missing_after
predicate already meets the bar, delete the whole stale-expression subsystem.

This script produces that deletion variant: the recheck becomes one plain
`infer_expr(source, scope, expected)` call, and the three functions go with it. No new
decision site is added, so the sparse fold's still_missing_after stays the single
authority (item 4).

Anchors were read from src/v1/04_infer.dag at HEAD, not inferred from the generated
mirror (the item-4 lesson: an inferred anchor silently matches nothing). Fail-closed: any
anchor that is not unique aborts with rc=4 and writes nothing.
"""
import sys, pathlib

STALE_FN = """fn typed_expr_is_stale(n: Node, scope: InferScope, shadowed: Map<String, Bool>) -> Bool {
  match n.expr_data {
    ExprVar { binding_kind: _ } =>
      let name = expr_var_name_at(texpr: n, source_indices: scope.type_env.source_indices)
      if map_has(shadowed, name) { false }
      else {
        match map_get(scope.locals, name) {
          Absent => false
          Present { value: binding } =>
            match n.inferred {
              Present { value: Resolved { node: rt } } =>
                !node_type_equals(left: rt, right: binding.resolved, source_indices: scope.type_env.source_indices)
              _ => true
            }
        }
      }
    ExprLambda =>
      let names = lambda_param_names_at(texpr: n, source_indices: scope.type_env.source_indices)
      let inner_shadowed = fold(names, init: shadowed, f: (acc, name) => map_insert(acc, name, true))
      typed_expr_is_stale(n: lambda_body(texpr: n), scope: scope, shadowed: inner_shadowed)
    _ => n.children |> any(c => typed_expr_is_stale(n: c, scope: scope, shadowed: shadowed))
  }
}

"""

RECHECK_HEAD = """fn recheck_expr(source: Node, prior: Node, scope: InferScope, expected: Node?, shadowed: Map<String, Bool>) -> InferResult {
  match source.expr_data {
"""

# recheck_expr's body is deleted up to the start of infer_planned_lambda_recheck.
LAMBDA_RECHECK_OLD = """  let ar_inferred = recheck_expr(source: source, prior: prior, scope: scope, expected: expected_context.expected, shadowed: shadowed)
"""
LAMBDA_RECHECK_NEW = """  // Item-5 deletion variant: plain full re-inference of the lambda argument against the
  // settled expected context. No staleness predicate, no structural walk -- one infer_expr.
  let ar_inferred = infer_expr(texpr: source, scope: scope, expected: expected_context.expected)
"""

def find_block(s, head, next_head):
    """Delete from `head` through the blank line before `next_head`."""
    i = s.find(head)
    if i < 0:
        return None, 0
    if s.count(head) != 1:
        return None, s.count(head)
    j = s.find(next_head, i)
    if j < 0:
        return None, -1
    return (i, j), 1

def main():
    check = "--check" in sys.argv
    path = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 and not sys.argv[1].startswith("--") else pathlib.Path("src/v1/04_infer.dag")
    if not path.exists():
        print("MISSING", path); return 2
    s = path.read_text()

    if s.count(LAMBDA_RECHECK_OLD) != 1:
        print("ANCHOR NOT UNIQUE (%d): lambda recheck call site" % s.count(LAMBDA_RECHECK_OLD)); return 4
    # stale fn: locate by signature line and delete through the blank line before recheck_expr
    stale_sig = "fn typed_expr_is_stale(n: Node, scope: InferScope, shadowed: Map<String, Bool>) -> Bool {"
    recheck_sig = "fn recheck_expr(source: Node, prior: Node, scope: InferScope, expected: Node?, shadowed: Map<String, Bool>) -> InferResult {"
    lambda_sig = "fn infer_planned_lambda_recheck(plan: CallArgumentPlan, expected_context: CallArgumentExpectedContext, first: ArgumentFirstReading, scope: InferScope) -> ArgInferResult {"
    for name, sig in (("typed_expr_is_stale", stale_sig), ("recheck_expr", recheck_sig), ("infer_planned_lambda_recheck", lambda_sig)):
        if s.count(sig) != 1:
            print("ANCHOR NOT UNIQUE (%d): %s signature" % (s.count(sig), name)); return 4
    i_stale = s.index(stale_sig); i_recheck = s.index(recheck_sig); i_lambda = s.index(lambda_sig)
    if not (i_stale < i_recheck < i_lambda):
        print("UNEXPECTED ORDER stale=%d recheck=%d lambda=%d" % (i_stale, i_recheck, i_lambda)); return 4

    out = s[:i_stale] + s[i_lambda:]
    out = out.replace(LAMBDA_RECHECK_OLD, LAMBDA_RECHECK_NEW)
    # `prior` existed only to feed the staleness walk; with it deleted the binding is
    # unused, so drop it as well rather than leaving dead state behind.
    for dead in ("  let prior = arg_value(n: first.result.typed_arg)\n",
                 "  let shadowed = empty_shadowed_names()\n"):
        # both existed only to feed the staleness walk; leaving them would be dead state
        if out.count(dead) == 1:
            out = out.replace(dead, "")
    # empty_shadowed_names is now unreferenced; it was introduced for this walk only, so
    # the deletion takes it too rather than leaving an orphan helper.
    orphan_sig = "fn empty_shadowed_names() -> Map<String, Bool> {"
    if out.count(orphan_sig) == 1 and "empty_shadowed_names()" not in out.replace(out[out.index(orphan_sig):out.index(orphan_sig)+200], ""):
        i = out.index(orphan_sig)
        j = out.index("\n}\n\n", i) + 3
        out = out[:i] + out[j:]
    for gone in ("typed_expr_is_stale", "recheck_expr("):
        if gone in out:
            print("RESIDUE after deletion: %s (count %d)" % (gone, out.count(gone))); return 4
    if out.count("infer_planned_lambda_recheck(") != 2:
        print("UNEXPECTED infer_planned_lambda_recheck reference count %d (want def + 1 call)" % out.count("infer_planned_lambda_recheck(")); return 4
    if check:
        print("DELETE-STALE APPLIES CLEANLY (check-only)"); return 0
    path.write_text(out)
    print("DELETE-STALE OK %s: stale-expression subsystem deleted, lambda recheck is one plain infer_expr" % path)
    return 0

if __name__ == "__main__":
    sys.exit(main())
