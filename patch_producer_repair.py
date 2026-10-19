#!/usr/bin/env python3
"""THE #14152 PRODUCER REPAIR -- src/v1/04_infer.dag, borrowed_callable_call_type.

MEASURED CAUSE (probe v16 + the static read the parent asked for, msg_2d4b0512; full chain in
PROBE17_READ.md): `fn empty_map<K, V>() -> Map<K, V>` has ONLY generic parameters, so
`value_params` is empty, the argument fold never runs, `unified.subst` is empty_map(), and
substitute_generics short-circuits with `if map_is_empty(subst) { n }` -- handing the CALLER'S
fold the callee's DECLARED return type literally, with children still named K and V. Those are
Map's own formal parameters, so a later call that binds the same variable to a concrete
Map<Int,String> refuses against them. 39 of the 46 Map-family rows are that one node.

THE REPAIR, per the parent: instantiate the declared generic return type with fresh unsettled
variables at each use, THROUGH THE INSTANTIATION PATH THE FOLD ALREADY USES -- do not add a new
one. That path is `unbound_owner_param_variables` (this file, 11728-11731), which the record-
literal instantiation already uses: fold the callee's own type-parameter names into
`map_insert(acc, g, type_variable_node(id: g))`. It is reused verbatim in shape. The only
substitution mechanism stays the existing `substitute_generics` at the return, now fed with a
non-empty subst so its short-circuit no longer applies.

WHY SEEDING THE FOLD'S INIT (rather than substituting afterwards) IS THE SAFE SHAPE:
 - for a slot an argument DOES solve, unify_generics writes over the seed: the seeded variable is
   caught by the FIRST arm, binding_is_unsettled_formal_annotation, because
   generic_node_mentions(TypeVariable{id:K}, "K") is true against generic_names=[K,V] -- so the
   settled actual replaces it. The common path (identity<T>(x: T) binding T=Int) is unchanged.
 - for a slot no argument solves, the variable survives, and substitute_generics_apply's BARE-NAME
   arm (mirror 26004-26011) replaces a childless name node with no same-name guard -- unlike the
   TypeVariable arm at 25987, which does have one. So `Map`'s children K,V become real
   TypeVariable nodes.
 - those children then satisfy binding_has_type_variable_leaf at its FIRST arm
   (`Present { value: TypeVariable { id: _ } } => true`, 7976-7978), which is exactly the
   'unsettled binding' predicate already approved for this fold. The family is then skipped by an
   arm that exists today. NO fold arm is added, widened, or reordered by this patch.

Guards below are hard failures, not warnings: an edit that lands twice, or not at all, or in the
wrong function, is the class of mistake that cost this lane a silent revert of four upstream
commits (probe v10). The patch refuses to write anything unless every assertion holds.
"""
import sys, pathlib, re

TARGET = "src/v1/04_infer.dag"

# the exact three lines to replace, taken from the head, with the function name above them so a
# moved edit into some other fold cannot pass
OLD = '''fn borrowed_callable_call_type(gnode: Node, typed_args: List<Node>, scope: InferScope) -> BorrowedCallType {
  let source_indices = scope.type_env.source_indices
  let base = callable_inferred(n: gnode)
  let tp_names = fn_type_param_names(item: gnode, source_indices: source_indices)
  if tp_names |> count == 0 {
    BorrowedCallType { ret: base, diagnostics: [] }
  } else {
    let value_params = gnode.params |> filter(p => !param_is_generic_decl(p: p, source_indices: source_indices))
    let unified = fold(value_params |> enumerate, init: UnifyGenericsAcc { subst: empty_map(), diagnostics: [] }, f: (acc, pair) =>'''

NEW = '''fn borrowed_callable_call_type(gnode: Node, typed_args: List<Node>, scope: InferScope) -> BorrowedCallType {
  let source_indices = scope.type_env.source_indices
  let base = callable_inferred(n: gnode)
  let tp_names = fn_type_param_names(item: gnode, source_indices: source_indices)
  if tp_names |> count == 0 {
    BorrowedCallType { ret: base, diagnostics: [] }
  } else {
    let value_params = gnode.params |> filter(p => !param_is_generic_decl(p: p, source_indices: source_indices))
    // A DECLARED GENERIC RETURN TYPE IS INSTANTIATED AT EACH USE, like every other generic call.
    // A callable whose parameters are ALL generic (std.algebra's `fn empty_map<K, V>() -> Map<K, V>`)
    // contributes no value parameter, so the fold below never runs, its subst stays empty, and
    // substitute_generics short-circuits on `if map_is_empty(subst) { n }` -- which returned the
    // callee's DECLARED return type literally, with children still named K and V. Those are Map's
    // OWN formal parameters, not this call's types, and the caller's fold then refused any later
    // binding of the same variable against them (`type variable A bound as
    // Map(Primitive(K),Primitive(V)) (first argument)`): 39 of the 46 rows in that family are that
    // one node, dag/std/algebra.dag:73644-73647, surfacing in 15 different callee modules.
    // The seed is the SAME shape the record-literal instantiation already uses for an
    // un-instantiated owner parameter (this file, unbound_owner_param_variables), so no new
    // instantiation mechanism is introduced. A slot an argument does solve is overwritten: the
    // seeded variable is the callee's own parameter name, so the fold's first arm
    // (binding_is_unsettled_formal_annotation) replaces it with the settled actual, leaving the
    // solved path exactly as it was. A slot nothing solves stays a variable, which is what
    // binding_has_type_variable_leaf already treats as unsettled.
    let unsettled_slot_variables = fold(tp_names, init: empty_map(), f: (acc, slot) =>
      map_insert(acc, slot, type_variable_node(id: slot))
    )
    let unified = fold(value_params |> enumerate, init: UnifyGenericsAcc { subst: unsettled_slot_variables, diagnostics: [] }, f: (acc, pair) =>'''

BANNED = [  # the patch must not smuggle in a probe, a new fold arm, or a new instantiation path
    "REBIND", "rebind16", "eprintln", "binding_is_generic_declaration_own_form",
    "substitute_generics_apply", "generic_declaration_application", "fn ",
]


def patch(root):
    p = pathlib.Path(root) / TARGET
    src = p.read_text(encoding="utf-8")
    n = src.count(OLD)
    if n == 0:
        if "unsettled_slot_variables" in src:
            print("ALREADY PATCHED (idempotent no-op)"); return 0
        i = src.index("fn borrowed_callable_call_type")
        print("OLD not found. Context around the function:\n" + src[i:i + 1400])
        return 41
    assert n == 1, "the anchor matches %d times; refusing to patch" % n
    out = src.replace(OLD, NEW, 1)
    # the change is exactly the seed; nothing else may move
    assert out.count("unsettled_slot_variables") == 2, "seed defined/used count wrong"
    assert "init: UnifyGenericsAcc { subst: unsettled_slot_variables" in out
    assert out.count("fn borrowed_callable_call_type") == src.count("fn borrowed_callable_call_type")
    assert out.count("borrowed_callable_call_type(") == src.count("borrowed_callable_call_type(")
    # brace balance must not move: an unbalanced .dag aborts regen with a diagnostic that is easy
    # to mistake for an unrelated one
    for a, b in (("{", "}"), ("(", ")")):
        assert out.count(a) - out.count(b) == src.count(a) - src.count(b), "unbalanced %s%s" % (a, b)
    # The inserted CODE (comments excluded -- they legitimately name `fn empty_map` and the
    # predicates they cite) must add no probe, no new definition and no second instantiation path.
    old_body = [l.strip() for l in OLD.splitlines()]
    added = [l for l in NEW.splitlines()
             if l.strip() and l.strip() not in old_body and not l.strip().startswith("//")]
    code = "\n".join(added)
    for bad in BANNED:
        if bad in code:
            raise AssertionError("inserted code contains %r: %s" % (bad, code[:200]))
    assert added, "nothing was inserted"
    print("inserted %d code line(s):" % len(added))
    for l in added:
        print("   +" + l.strip()[:110])
    if len(sys.argv) > 2 and sys.argv[2] == "write":
        p.write_text(out, encoding="utf-8")
        print("PATCHED %s (%d -> %d bytes)" % (TARGET, len(src), len(out)))
    else:
        print("dry-run ok: %d -> %d bytes" % (len(src), len(out)))
    return 0


if __name__ == "__main__":
    sys.exit(patch(sys.argv[1] if len(sys.argv) > 1 else "."))
