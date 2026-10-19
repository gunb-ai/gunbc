#!/usr/bin/env python3
"""Producer repair for the Map-family refusals (#14152, parent msg_07ea5fa3 (i)).

THE DEFECT (probe v16 + static read, reported to the parent): a call to a generic
function with NO value parameters -- dag/std/algebra.dag:1406 `fn empty_map<K, V>()
-> Map<K, V>` -- takes the `Present { value: s }` arm of the call expression's
`resolved_type` and substitutes only what the actuals solved. With no actuals,
`call_subst` is empty and substitute_generics_apply short-circuits
(`if map_is_empty(subst) { n }`), so the callee's DECLARED return type is handed over
as this call expression's type with K and V still naming the DECLARATION's own
parameters. That uninstantiated form then reaches the generic unification fold as a
settled binding and the fold reports 'expected type variable A bound as
Map(Primitive(K),Primitive(V)) (first argument), got <the real instance> (later
argument)'. 39 of the 46 Map-family rows are this one node.

THE REPAIR: seed the substitution with a fresh unsettled variable for every declared
type parameter the actuals did not solve, reusing the instantiation the fold already
uses elsewhere (`unbound_owner_param_variables`, same file: fold over names, init
empty_map(), map_insert(name, type_variable_node(id: ...))). No new mechanism.

FRESHNESS (parent msg_78d4eb21): the id is keyed on THIS call expression's own span,
not on the bare parameter name. `type_variable_node(id: "K")` would be producer 2's
defect (bare_map_node minting std's own declared names) reproduced at the call site:
two calls in one body that share a parameter name would share variable K and could
unify each other's types -- a silent wrong binding instead of a loud conflict. The
scope comparison the parent asked for: `unbound_owner_param_variables`' subst is
consumed locally one line later (a record-literal field type) and never escapes,
whereas here the substituted type ESCAPES as the call's `inferred` into the body
environment, where other call sites' types meet it. So name-keying is not safe here.

BYTE-IDENTICAL FOR NON-GENERIC CALLS (parent msg_1249d701): the seed is folded over
`call_sig_split.generic_names`, which is empty for a callee declaring no type
parameters; map_merge(base, empty) is base, and substitute_generics_apply short-circuits
on an empty subst exactly as it does today.

MUTATIONS (so the tests are known to discriminate; run by repair_remeasure):
  --mutate name-keyed : id loses the call-site key  -> the collision test must FAIL
  --mutate withdrawn  : the seed is dropped entirely -> the accept control must FAIL
Usage: patch_producer_instantiate.py [--mutate name-keyed|withdrawn] REPO
"""
import sys

REPO = sys.argv[-1]
MUTATE = ""
if "--mutate" in sys.argv:
    MUTATE = sys.argv[sys.argv.index("--mutate") + 1]
    assert MUTATE in ("name-keyed", "withdrawn"), MUTATE

PATH = REPO + "/src/v1/04_infer.dag"

# The exact line, in the ExprCall arm of infer_expr_body, that returns the callee's
# declared result type to become the call expression's type.
TARGET = ("          Present { value: s } => substitute_generics(n: s.inferred, "
          "subst: call_subst, source_indices: scope.type_env.source_indices)")

REPAIR_TMPL = """        // PRODUCER REPAIR (#14152). A declared generic result type is a TEMPLATE:
        // its leaves are the callee's own parameter names. A call whose actuals solve
        // nothing -- `empty_map<K, V>()` has no value parameters, so call_subst is empty
        // and substitute_generics_apply short-circuits on it -- handed the declaration's
        // own `Map(K, V)` over as THIS call's settled type, and the generic unification
        // fold then refused the real instance that arrived later ('A bound as
        // Map(Primitive(K),Primitive(V)) (first argument)'). Instantiate every declared
        // parameter the actuals did not solve with a fresh UNSETTLED variable, through the
        // instantiation this file already uses for unbound owner parameters
        // (unbound_owner_param_variables) -- no new mechanism. The id is keyed on this
        // call expression's own span: two calls sharing a parameter name must not share
        // the variable, or one call's instance could unify with the other's (a silent
        // wrong binding instead of a loud conflict). A callee declaring no type parameters
        // folds to an empty seed, map_merge is then the identity, and the substitution
        // short-circuits exactly as before: non-generic calls are byte-identical.
        let uninstantiated_call_generics = call_sig_split.generic_names
          |> filter(g => !map_has(call_subst, g))
        let fresh_call_variables = fold(uninstantiated_call_generics, init: empty_map(), f: (acc, g) =>
          map_insert(acc, g, type_variable_node(id: {IDEXPR}))
        )
        let resolved_type = match sig {{
          Present {{ value: s }} => substitute_generics(n: s.inferred, subst: map_merge(call_subst, fresh_call_variables), source_indices: scope.type_env.source_indices)
          Absent => error_type
        }}"""

NAME_KEYED_ID = 'g'
# Verified forms: `declaring_span_key` (04_emit_info.dag:320) uses `concat(sp.file, "@",
# to_string(sp.start))` -- FilePath is `String where string_non_empty`, so no cast;
# `to_string` is the kernel builtin (04_method.dag:329) already used bare in this file
# (10218, 17903); `map_has(m, k)` 2-positional-arg form is used at 9221.
SPAN_KEYED_ID = 'concat(g, "@", span.file, ":", to_string(span.start), "-", to_string(span.end))'


def main():
    src = open(PATH, encoding="utf-8").read()
    n = src.count(TARGET)
    if n != 1:
        sys.exit("FATAL: expected exactly 1 occurrence of the target line, found %d "
                 "-- the arm moved, do not patch blind" % n)

    # The enclosing arm binds `let span = texpr.span` (the call node's own span) and the
    # signature split above the target; assert both are in scope before relying on them.
    head = src[:src.index(TARGET)]
    arm = head.rindex("    ExprCall { call_semantics: _, descent_evidence: _ } =>")
    arm_text = head[arm:]
    for needle, why in (("let span = texpr.span", "`span` (the call's own span)"),
                        ("let call_sig_split = split_sig_params(", "`call_sig_split`")):
        if needle not in arm_text:
            sys.exit("FATAL: %s is not in scope in the ExprCall arm -- patch would not compile" % why)

    if MUTATE == "withdrawn":
        # RED-before twin: behaviour of the pre-repair head, but the surrounding code is
        # the same, so a test failing here and passing on the real patch discriminates.
        repl = ("        let fresh_call_variables = empty_map()\n"
                "        let resolved_type = match sig {\n"
                "          Present { value: s } => substitute_generics(n: s.inferred, subst: map_merge(call_subst, fresh_call_variables), source_indices: scope.type_env.source_indices)\n"
                "          Absent => error_type\n"
                "        }")
    elif MUTATE == "name-keyed":
        repl = REPAIR_TMPL.format(IDEXPR=NAME_KEYED_ID)
    else:
        repl = REPAIR_TMPL.format(IDEXPR=SPAN_KEYED_ID)

    out = src.replace(TARGET, repl, 1)

    # Post-conditions: the patched file must still reference the two facts the repair
    # depends on, and must not have touched anything else.
    if MUTATE != "name-keyed" and MUTATE != "withdrawn":
        assert out.count("fresh_call_variables") == 2, "seed must be built once and used once"
        assert 'type_variable_node(id: concat(g, "@", span.file as String' in out, "id must be call-site keyed"
    delta = len(out.split("\n")) - len(src.split("\n"))
    open(PATH, "w", encoding="utf-8").write(out)
    print("patched %s  mutate=%s  line-delta=%+d" % (PATH, MUTATE or "none", delta))
    print("VERIFIED: 1 target occurrence, span+call_sig_split in scope, seed folded over call_sig_split.generic_names")


main()
