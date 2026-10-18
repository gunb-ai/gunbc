#!/usr/bin/env python3
"""Analyse REBIND16| rows: WHICH PRODUCER hands the fold a Map whose args are builtin Map's own parameters?

stern-bat-401's ruling (msg_840e0c82) and DESIGN.md SS6b: do not widen the predicate and do not add a
head-declared-parameters accessor. Name the producer per row -- by production call or expression identity --
and report whether it is ONE producer or SEVERAL, before building anything.

PRE-REGISTERED OUTCOMES, written before the rows were seen, so the measurement can falsify a hypothesis
instead of selecting a comfortable story:

  O1  Every Map(K,V) row reports bocc/occ on the K and V children pointing at <kernel:Map> (or any
      file that is not a user .dag).      => the producer is the KERNEL's own Map shape: these are the
      builtin's slot leaves travelling as arguments. That is a substrate fact about builtin generics in
      std, i.e. the model-first prerequisite stern-bat-401 said he would dispatch separately. #14152 waits.

  O2  The K,V children carry spans in USER .dag files, clustered on few distinct (file, span) pairs.
      => the producer is a source construct at those positions (an empty-map literal, or a declared type
      read without instantiation). Then the boundary is that construct, and the fix is at ITS producer.

  O3  The occurrence_identity of the K/V children is OccurrenceSynthetic / OccurrenceMinted while the
      bound node itself is parsed (ident_span present). => a minting routine made them; name it from
      decl/expr columns and it is the earliest candidate boundary.

  O4  Distinct producers are many (say >10 distinct (file,span) clusters) AND they are spread across
      unrelated modules. => there is no single producer; the fold is being handed the generic FORM
      generally, which means it is a binding-policy question, not a producer bug. Report that and stop.

  O5  Rows are zero, or the bocc/decl/expr columns are uniformly empty. => THE PROBE DID NOT MEASURE, which
      is not an answer. Do not conclude anything from empty columns; fix the probe.

A count of DISTINCT producers is the deliverable, so it is printed first.
"""
import re
import sys
from collections import Counter

KEYS = ("module", "bind", "formal", "bhead", "bspan", "bident", "bocc", "bdecl", "bexpr",
        "bparams", "bleaves", "aleaves", "gnames", "ghit", "ownBound", "achildren", "aexprs")


def field(row, key):
    m = re.search(r"\|" + re.escape(key) + r"=([^|]*)", row)
    if m is None:
        return None
    return m.group(1)


def is_map_family(row):
    """Bound side's childless leaves are exactly the builtin Map's own parameters."""
    v = field(row, "bleaves") or ""
    leaves = {x for x in v.split(",") if x}
    return bool(leaves) and leaves <= {"K", "V"}


def main(path):
    rows = [l for l in open(path, encoding="utf-8", errors="replace").read().split("\n")
            if l.startswith("REBIND16|")]
    if not rows:
        print("NO ROWS (O5): the probe emitted nothing. Not a measurement of zero.")
        return 1
    for k in ("bocc", "bspan", "achildren"):
        if any(field(r, k) is None for r in rows):
            print(f"EXTRACTOR/PROBE BROKEN: field '{k}' absent -- refusing to report (O5).")
            return 2
    print(f"rows={len(rows)}")

    fam = [r for r in rows if is_map_family(r)]
    print(f"Map(K,V) family rows={len(fam)}  (bound childless leaves ⊆ {{K,V}})")
    if not fam:
        print("  no family rows in this corpus: the producer question is UNANSWERED here (O5), not 'no producer'.")
        return 0

    # THE DELIVERABLE: distinct producers, three ways, because 'one producer or several' is the question.
    print("\n== PRODUCER IDENTITY, three independent cuts ==")
    # (a) the bound node's own span file
    files = Counter((field(r, "bspan") or "").split(":")[0] for r in fam)
    print(f"(a) bound-node span FILE: {len(files)} distinct")
    for f, n in files.most_common(12):
        print(f"    {n:4d}  {f}")
    # (b) child leaves' spans -- where did K and V come from
    child_spans = Counter()
    for r in fam:
        for entry in (field(r, "achildren") or "").split(","):
            if "@" in entry:
                child_spans[entry.split("@", 1)[1].split("/")[0]] += 1
    print(f"(b) CHILD leaf spans: {len(child_spans)} distinct")
    for f, n in child_spans.most_common(15):
        print(f"    {n:4d}  {f}")
    # (c) occurrence identity of the children, parsed out of the achildren strings
    occ = Counter()
    for r in fam:
        for entry in (field(r, "achildren") or "").split(","):
            m = re.search(r"/occ=([A-Za-z]+)", entry)
            if m:
                occ[m.group(1)] += 1
    print(f"(c) CHILD occurrence_identity kinds: {dict(occ)}")
    bocc = Counter((field(r, "bocc") or "")[:40] for r in fam)
    print(f"(d) bound-node occurrence_identity: {dict(bocc.most_common(6))}")
    decls = Counter((field(r, "bdecl") or "")[:60] for r in fam)
    print(f"(e) bound-node declaration (Debug text): {len(decls)} distinct; top: "
          + ", ".join(f"{k}x{v}" for k, v in decls.most_common(4)))
    exprs = Counter((field(r, "bexpr") or "")[:40] for r in fam)
    print(f"(f) bound-node expr_data (Debug text): {dict(exprs.most_common(6))}")
    kid_exprs = Counter()
    for r in fam:
        for e in (field(r, "aexprs") or "").split(","):
            if e:
                kid_exprs[e[:34]] += 1
    print(f"(g) CHILD expr_data kinds: {dict(kid_exprs.most_common(8))}")

    # module spread, for O4
    mods = Counter(field(r, "module") for r in fam)
    print(f"\nmodule spread within the family: {len(mods)} modules")
    for m, n in mods.most_common(10):
        print(f"    {n:4d}  {m}")

    print("\n== OUTCOME MAPPING (pre-registered; report which one the data selects) ==")
    kern = sum(n for f, n in files.items() if f.startswith("<kernel"))
    if kern == len(fam) and len(child_spans) and all("kernel" in c for c in child_spans):
        print("  O1: every leaf points at a kernel location -> builtin-generic substrate fact; std prerequisite.")
    elif len(child_spans) <= 10:
        print(f"  O2: leaves cluster on {len(child_spans)} source positions -> producer is a source construct.")
    elif len(child_spans) > 10:
        print(f"  O4: {len(child_spans)} distinct leaf positions -> no single producer; binding-policy question.")
    minted = sum(n for k, n in occ.items() if "Synthetic" in k or "Minted" in k)
    if minted and minted >= sum(occ.values()):
        print("  O3: all children are synthetic/minted -> name the minting routine from decl/expr columns.")
    emptyish = sum(1 for r in fam if not (field(r, "bdecl") or "").strip("{}\"' "))
    if emptyish == len(fam):
        print("  O5: declaration column uniformly empty -> probe did not measure; do not conclude.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
