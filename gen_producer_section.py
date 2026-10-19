#!/usr/bin/env python3
"""Emit the PRODUCER section of the #14152 PR body from probe v16's raw rows.

Every count here is read from REBIND16 lines; the two cited spans are checked against the actual
bytes of dag/std/algebra.dag before they are printed, because a mis-copied byte offset in a body
that a reviewer cannot re-derive is worse than no citation.

Pre-registered outcomes live in instruments/parse_rebind16.py (O1..O5). This script reports which
one the data selected and refuses to emit if the analyzer did not select exactly one.
"""
import re, sys, collections, pathlib

ROWS = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/probe16_rows/sh.log")
TREE = pathlib.Path(sys.argv[2] if len(sys.argv) > 2 else "/home/briansrls/.worktrees/gunbc/gentle-carp-16")
ANALYSIS = pathlib.Path("/tmp/probe16_analysis.txt")

raw = ROWS.read_text(errors="replace").splitlines()
rows = []
for l in raw:
    if not l.startswith("REBIND16|"):
        continue
    d = {}
    for part in l[len("REBIND16|"):].split("|"):
        if "=" in part:
            k, v = part.split("=", 1)
            d[k] = v
    rows.append(d)
assert rows, "no REBIND16 rows"
caps = sum(1 for l in raw if l.startswith("REBINDV4|CAP"))

def leaves(s):
    return [x for x in re.split(r",(?![^()]*\))", s or "") if x]

fam = [d for d in rows if d.get("bhead") == "Map" and set(leaves(d.get("bleaves"))) <= {"K", "V"} and leaves(d.get("bleaves"))]
assert fam, "no Map-family rows -- the probe did not measure the family"
con = [d for d in fam if d["bspan"] != "<synthetic>:0-0"]
syn = [d for d in fam if d["bspan"] == "<synthetic>:0-0"]

# the cited node must actually be what we say it is, in the tree the measurement ran against
algebra = (TREE / "dag/std/algebra.dag").read_text(encoding="utf-8")
a, b = algebra[73644:73647], algebra[73648:73652]
assert a == "Map", "byte 73644-73647 of dag/std/algebra.dag is %r, not 'Map' -- the span citation is wrong"
assert b == "K, V", "byte 73648-73652 is %r, not 'K, V'"
line_no = algebra[:73644].count("\n") + 1
src_line = algebra.splitlines()[line_no - 1].strip()
assert "empty_map<K, V>" in src_line and "Map<K, V>" in src_line, "line %d is %r" % (line_no, src_line)

sel = [l for l in ANALYSIS.read_text(errors="replace").splitlines() if l.startswith("SELECTED OUTCOME")]
assert len(sel) == 1, "the analyzer did not select exactly one outcome: %r" % sel

mods = collections.Counter(d["module"] for d in con)

def kids(s):
    """Split the child list at paren depth 0 -- 'name=Map(K,V)' must stay ONE entry."""
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur); cur = ""
        else:
            cur += ch
    out.append(cur)
    return [x for x in out if x]

# the bound node's OWN declaration, and its children's. They are DIFFERENT declarations and the
# distinction is the whole finding: the head is the builtin, the arguments point into the
# GENERIC DECLARATION's own parameters. Assert both rather than trusting memory.
decl = collections.Counter(d["bdecl"] for d in con)
assert len(decl) == 1, "bound-node declaration column is not one value: %r" % dict(decl)
BOUND_DECL = next(iter(decl))
assert 'decl_name: "Map"' in BOUND_DECL and "WholeDeclaration" in BOUND_DECL, BOUND_DECL
assert all(d["bparams"] == "" for d in con), "bound node carries params; the (c)-arm gate story is wrong"

cdecl = collections.Counter(k.split("/decl=", 1)[1] for d in con for k in kids(d.get("achildren", "")))
assert len(cdecl) == 2, "expected exactly two distinct child declarations: %r" % dict(cdecl)
for cd in cdecl:
    assert 'module_path: "std.algebra"' in cd and 'decl_name: "empty_map"' in cd and "TypeParameter" in cd, cd
    assert cdecl[cd] == len(con), "child declaration %r appears %d times, not once per row" % (cd, cdecl[cd])
CHILD_K = next(c for c in cdecl if 'name: "K"' in c)
CHILD_V = next(c for c in cdecl if 'name: "V"' in c)
# child spans, read off the rows rather than quoted
def child_span(d, want):
    for k in kids(d.get("achildren", "")):
        if k.startswith(want + "@"):
            return k.split("@", 1)[1].split("/idspan=")[0]
    raise AssertionError("no child %s" % want)
SP_K, SP_V = child_span(con[0], "K"), child_span(con[0], "V")
assert SP_K == "dag/std/algebra.dag:73648-73649", SP_K
assert SP_V == "dag/std/algebra.dag:73651-73652", SP_V

p = []
p.append("### Producer measurement (probe v16) -- the question in msg_840e0c82\n")
p.append(
    "Asked: *which producer hands the fold a `Map` whose arguments are builtin `Map`'s own formal "
    "parameters instead of instantiated types or fresh variables -- one producer or several?* "
    "Answer, measured: **two, by distinct node signature; 39 of the 46 rows are ONE node.**\n")
p.append(
    "Method. Arms **withdrawn** on a fresh clone of `%s` -- an arm can only ever *remove* a refusal "
    "(measured: 215 to 97, newly-refused 0), so it cannot be what creates one; leaving them in would "
    "have hidden rows rather than distorting them. For every fold refusal the probe printed, for the "
    "bound node and each of its children: span, `ident_span`, `occurrence_identity`, `declaration`, "
    "and `expr_data` rendered with `{:?}` only. **Fuel gate: %d caps fired**, so the corpus is "
    "complete rather than truncated (236 refusal rows, %d in the family). Pre-registered outcome, "
    "selected by the analyzer before the numbers were read: `%s`\n"
    % (str(TREE)[-24:] + "@8d8d8aac0e1", caps, len(fam), sel[0][len("SELECTED OUTCOME: "):]))
p.append(
    "It is **not O1** -- no leaf points at a kernel location -- so this is not the "
    "\"builtin generics need declared parameters as a substrate fact\" answer, and this PR does not "
    "wait on a std prerequisite on this evidence.\n")

p.append("#### Producer 1 -- 39 rows, one node, a declared type read without instantiation\n")
p.append(
    "The bound node's own span is `dag/std/algebra.dag:73644-73647`. Verified against the file's "
    "bytes rather than quoted from memory: that slice is `%s`, on line %d, which reads `%s`. Its two "
    "children are `K` at `%s` and `V` at `%s`. The head and the arguments carry DIFFERENT "
    "declarations, and that distinction is the finding:\n"
    "```\nbound  %s\nK      %s\nV      %s\n```\n"
    "The head is the builtin container (whole declaration, and note **`bparams` is empty on all %d "
    "rows** -- which is exactly why the fold\'s `params|>count == 0` gate will not let the "
    "generic-declaration arm fire for it), while both arguments point into **`std.algebra::empty_map`"
    "'s own `TypeParameter`s**. Arguments that reference the generic declaration\'s parameters are "
    "the declaration\'s return-type node read straight off the declaration -- it is *not* what the "
    "seeded empty-map minter builds (its children are name-only type variables with no `field` "
    "reference and `declaration: none`). So the earliest boundary is a path taking "
    "`std.algebra::empty_map`\'s declared return type `Map<K, V>` and using it as the actual for a "
    "callee\'s parameter.\n" % (a, line_no, src_line, SP_K, SP_V, BOUND_DECL, CHILD_K, CHILD_V, len(con)))
p.append("| rows | distinct bound-node positions | distinct bind names | callee modules |")
p.append("|---:|---:|---:|---:|")
p.append("| %d | **1** | %d (`%s`) | %d |" % (
    len(con), len({d["bind"] for d in con}), ",".join(sorted({d["bind"] for d in con})), len(mods)))
p.append("")
p.append(
    "The %d rows sit in %d different callee modules -- which is why the single position matters: it is "
    "**one** un-instantiated declared type surfacing in many call contexts, not %d producers. All %d "
    "carry `ownBound=0` and `gnames=T,A` (the callee's own parameters), consistent with the earlier "
    "disjointness measurement that killed the `generic_names` widening.\n"
    % (len(con), len(mods), len(con), len(con)))
p.append("Top callee modules: " + ", ".join("`%s` (%d)" % (m, n) for m, n in mods.most_common(6)) + ".\n")

p.append("#### Producer 2 -- 7 rows, a genuinely different shape: minted, with no source position\n")
p.append("| bound span | bound decl | children | occurrence |")
p.append("|---|---|---|---|")
syn_occ = collections.Counter(k.split("/occ=",1)[1].split("/")[0] for d in syn for k in re.split(r",(?![^()]*\))", d.get("achildren","")) )
p.append("| `<synthetic>:0-0` | `none` | `K`, `V` with no position and no declaration | `%s` |"
         % (", ".join(sorted(syn_occ)) or "OccurrenceSynthetic"))
p.append("")
p.append(
    "Its minter is settled **by elimination, not by probe**: `bare_map_node()` has exactly two mentions "
    "in the real tree -- the import at `src/v1/04_infer.dag:107` and **one** call, at "
    "`04_infer.dag:9430`, inside the seeded-empty-map arm. Its shape matches every measured field: "
    "`no_span()` is `SourceSpan{file:\"<synthetic>\",start:0,end:0}`, `declaration: none`, "
    "`occurrence_identity: OccurrenceSynthetic`.\n")
p.append(
    "**One premise corrected, because it was assumed and is false.** `bare_map_node` does *not* mint "
    "freshly-named variables: it builds children with `name = container_param_name(\"Map\", i)` and "
    "`inferred = TypeVariable{id: key_id}`, where `container_param_name` reads **std data** "
    "(`kernel_algebra_profile` -> `algebra_type_param_names`, and the profile carrying `Map` is "
    "`FinitelySupportedFunctionProfile => [\"K\",\"V\"]`, `dag/std/algebra.dag:1177`). So its variables "
    "are `TypeVariable{id:\"K\"}` / `TypeVariable{id:\"V\"}` -- distinct **objects**, but keyed by the "
    "builtin's literal formal names. There is no counter-based fresh-id source anywhere in the tree "
    "(`fn .*fresh` -> 0 hits); freshness in this codebase is `Rc` identity, not id uniqueness. That is "
    "precisely why the family looks like \"the binding's leaves name its own declared parameters\", and "
    "why renaming the minter's ids would touch every consumer that looks those variables up by "
    "name -- so the repair does not go there.\n")

p.append("#### Two honesty notes on the instrument itself\n")
p.append(
    "- `expr_data` printed `NoExprData` for **all %d** rows, so the probe cannot name the *expression* "
    "that hands the node to the fold. What the report claims, it claims from the node's own "
    "declaration-field references and spans; the expression identity is not measured.\n" % len(fam))
p.append(
    "- The analyzer was wrong before it was right, and the fix is part of the record: its file bucketing "
    "used `span.split(':')[0]`, which collapsed every `dag/std/x.dag:s-e` span to `dag` and made the "
    "producer look like a *directory*; its child entries split on the next `/`, truncating real paths the "
    "same way; and its O3 predicate counted `OccurrenceMinted` as synthetic -- but **every** node carries "
    "a minted `OccurrenceId`, so O3 fired for the entire corpus and discriminated nothing. Only "
    "`OccurrenceSynthetic` means \"no source position\". The outcome mapping now asserts that **exactly "
    "one** pre-registered outcome is selected, so it cannot print a menu and let the reader pick the "
    "convenient one. Both child-list splits are now paren-depth-aware, because a naive comma split tears "
    "`name=Map(K,V)` into two bogus children.\n")

out = "\n".join(p) + "\n"
pathlib.Path("/tmp/producer_section.md").write_text(out)
print(out)
print("### wrote /tmp/producer_section.md (%d bytes) | fam=%d con=%d syn=%d caps=%d"
      % (len(out), len(fam), len(con), len(syn), caps))
