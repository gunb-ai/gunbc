#!/usr/bin/env python3
"""Producer repair for the Map-family refusals (#14152, parent msg_07ea5fa3 (i)).

THE DEFECT (probe v16 + static read, reported to the parent): a call to a generic function
with NO value parameters -- dag/std/algebra.dag:1406 `fn empty_map<K, V>() -> Map<K, V>`,
1489 corpus calls, all spelled bare -- reaches the call arm's
`Present { value: s }` branch and substitutes only what the actuals solved. With no actuals
`call_subst` is empty and substitute_generics_apply short-circuits
(`if map_is_empty(subst) { n }`), so the callee's DECLARED return type becomes this call
expression's type with K and V still naming the DECLARATION's own parameters. That
uninstantiated form then reaches the generic unification fold as if settled, and the fold
refuses the real instance that arrives later: "expected type variable A bound as
Map(Primitive(K),Primitive(V)) (first argument), got <the real instance> (later argument)".
39 of the 46 Map-family rows are this one node.

THE REPAIR: seed the substitution with a fresh UNSETTLED variable for every declared type
parameter the actuals did not solve, reusing the instantiation this file already uses
(unbound_owner_param_variables: fold over names, init empty_map(), map_insert(name,
type_variable_node(id: ...))). No new mechanism.

FRESHNESS (parent msg_78d4eb21): the id is keyed on THIS call expression's own span, not on
the bare parameter name. `type_variable_node(id: "K")` would be producer 2's defect
(bare_map_node minting std's own declared names) reproduced at the call site: two calls in
one body sharing a parameter name would share the variable and could unify each other's
types -- a silent wrong binding instead of a loud conflict. The scope comparison the parent
asked for: unbound_owner_param_variables' subst is consumed locally one line later (a
record-literal field type) and never escapes, whereas here the substituted type ESCAPES as
the call's `inferred` into the body environment, where other call sites' types meet it. So
name-keying is not safe here.

BYTE-IDENTICAL FOR NON-GENERIC CALLS (parent msg_1249d701): the seed folds over
`call_sig_split.generic_names`, empty for a callee declaring no type parameters; map_merge
with an empty overlay is the identity, and substitute_generics_apply then short-circuits
exactly as it does today.

MUTATIONS (so the tests are known to discriminate; run by repair_measure_14152.sh):
  --mutate name-keyed : the id loses the call-site key   -> the collision row must FAIL
  --mutate withdrawn  : the seed is dropped              -> the accept control must FAIL

Usage: patch_producer_instantiate.py [--mutate name-keyed|withdrawn] REPO
"""
import sys

MUTATE = ""
args = sys.argv[1:]
if "--mutate" in args:
    i = args.index("--mutate")
    MUTATE = args[i + 1]
    assert MUTATE in ("name-keyed", "withdrawn"), MUTATE
    args = args[:i] + args[i + 2:]
REPO = args[-1]

PATH = REPO + "/src/v1/04_infer.dag"

# The complete statement, in the ExprCall arm of infer_expr_body, that turns the callee's
# declared result type into the call expression's type. The WHOLE statement is the anchor:
# anchoring on the arm line alone and re-emitting the `let` puts a `let` inside
# `match sig { ... }`, which is not a match arm -- the parser said exactly that
# (`parse-failed at src/v1/04_infer.dag:9287:13: expected FatArrow, found Ident`) and the
# regeneration refused, leaving the mirror byte-identical. That cost one dispatch.
TARGET = """        let resolved_type = match sig {
          Present { value: s } => substitute_generics(n: s.inferred, subst: call_subst, source_indices: scope.type_env.source_indices)
          Absent => error_type
        }"""

REPLACED = """        let resolved_type = match sig {
          Present { value: s } => substitute_generics(n: s.inferred, subst: map_merge(call_subst, fresh_call_variables), source_indices: scope.type_env.source_indices)
          Absent => error_type
        }"""

# NO `//` COMMENT INSIDE THE FUNCTION BODY. Measured, not assumed: a comment line here is
# diagnosed `source annotation sits inside a declaration body. Only module-item grain is
# modeled` and it is a HARD diagnostic -- my 15-line block took this file's hard-diagnostic
# count from 241 to 256, which would have polluted the two-closure gate and the self-host
# count with something that is not a type error. The explanation lives in the commit message
# and the PR body instead, where nothing measures it.
COMMENT = ""

SEED = COMMENT + """
        let uninstantiated_call_generics = call_sig_split.generic_names
          |> filter(g => !map_has(call_subst, g))
        let fresh_call_variables = fold(uninstantiated_call_generics, init: empty_map(), f: (acc, g) =>
          map_insert(acc, g, type_variable_node(id: __ID__)))
"""

NAME_KEYED_ID = "g"
# Verified forms: declaring_span_key (04_emit_info.dag:320) uses concat(sp.file, "@",
# to_string(sp.start)) -- FilePath is `String where string_non_empty`, so no cast; to_string
# is the kernel builtin (04_method.dag:329) already used bare in this file (10218, 17903);
# map_has(m, k) is the 2-positional-arg form used at 04_infer.dag:9221.
SPAN_KEYED_ID = 'concat(g, "@", span.file, ":", to_string(span.start), "-", to_string(span.end))'


def main():
    src = open(PATH, encoding="utf-8").read()
    n = src.count(TARGET)
    if n != 1:
        sys.exit("FATAL: expected exactly 1 occurrence of the target statement, found %d "
                 "-- the arm moved, do not patch blind" % n)

    # The enclosing arm must bind the two facts the repair relies on. Check them in the ARM,
    # not just somewhere in the file: reading the branch without the guard that reaches it is
    # what made my first repair inert.
    head = src[:src.index(TARGET)]
    arm_start = head.rindex("    ExprCall { call_semantics: _, descent_evidence: _ } =>")
    arm = head[arm_start:]
    for needle, why in (("let span = texpr.span", "`span` (the call's own span)"),
                        ("let call_sig_split = split_sig_params(", "`call_sig_split`")):
        if needle not in arm:
            sys.exit("FATAL: %s is not in scope in the ExprCall arm -- patch would not compile" % why)

    if MUTATE == "withdrawn":
        # The pre-repair behaviour expressed in the new code, so a row that passes here is
        # testing the surrounding code and not the seed.
        repl = ("        let fresh_call_variables = empty_map()\n" + REPLACED)
    elif MUTATE == "name-keyed":
        repl = SEED.replace("__ID__", NAME_KEYED_ID) + REPLACED
    else:
        repl = SEED.replace("__ID__", SPAN_KEYED_ID) + REPLACED

    out = src.replace(TARGET, repl, 1)

    # STRUCTURAL GATES -- each of these is a bug this session actually paid a dispatch for.
    assert "let " not in repl.split("let resolved_type")[1].split("\n        }")[0] \
        or True  # (the match body is checked below, on the merged text)
    stmt_at = out.index("let resolved_type = match sig {")
    body = out[stmt_at:]
    body = body[:body.index("\n        }")]
    if "let " in body[len("let resolved_type = match sig {"):]:
        sys.exit("FATAL: the match body contains a `let` -- that is not a match arm and the "
                 "generator will refuse to parse it")
    if out.count("{") - src.count("{") != out.count("}") - src.count("}"):
        sys.exit("FATAL: braces are no longer balanced")
    if MUTATE == "":
        if out.count("fresh_call_variables") != 2:
            sys.exit("FATAL: the seed must be built once and used once (found %d)"
                     % out.count("fresh_call_variables"))
        if "type_variable_node(id: concat(g, \"@\", span.file" not in out:
            sys.exit("FATAL: the fresh id must be call-site keyed")
        if out.index("let fresh_call_variables") > stmt_at:
            sys.exit("FATAL: the seed must precede the statement that consumes it")

    delta = len(out.split("\n")) - len(src.split("\n"))
    open(PATH, "w", encoding="utf-8").write(out)
    print("patched %s  mutate=%s  line-delta=%+d" % (PATH, MUTATE or "none", delta))
    print("VERIFIED: unique anchor, span+call_sig_split in the arm, match body arm-only, "
          "braces balanced, seed precedes its consumer")


main()
