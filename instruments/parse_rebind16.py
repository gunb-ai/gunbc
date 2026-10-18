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


def top_split(s, sep=","):
    """Split on `sep` only at paren depth 0. A naive s.split(',') tears a child entry
    whose name is itself a printed type -- 'name=Map(K,V)' becomes two bogus entries --
    which under-reports and mislabels the children. This instrument answers 'one
    producer or several', so the split has to be correct, not approximate."""
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch == "(" :
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == sep and depth == 0:
            out.append(cur); cur = ""
        else:
            cur += ch
    out.append(cur)
    return [x for x in out if x != ""]


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
    # strip the trailing :line or :line-line from a span WITHOUT touching the path.
    # An earlier version used split(":")[0], which collapsed every dag/std/x.dag:s-e span to
    # "dag" and made the producer look like one directory. Verified against the raw rows.
    def path_of(span):
        return re.sub(r":[0-9]+(-[0-9]+)?(:[0-9]+(-[0-9]+)?)?$", "", span or "")
    files = Counter(path_of(field(r, "bspan")) for r in fam)
    print(f"(a) bound-node span FILE: {len(files)} distinct")
    for f, n in files.most_common(12):
        print(f"    {n:4d}  {f}")
    # (b) child leaves' spans -- where did K and V come from
    child_spans = Counter()
    for r in fam:
        for entry in top_split(field(r, "achildren") or ""):
            if "@" in entry:
                # entry is `name@SPAN/idspan=IDSPAN/occ=OCC/decl=DECL`. Anchoring on the field
                # NAME matters: splitting on the next '/' truncated a real path to its first
                # directory and made 39 std/algebra.dag children read as one file called "dag".
                seg = entry.split("@", 1)[1]
                seg = seg.split("/idspan=", 1)[0]
                child_spans[path_of(seg)] += 1
    print(f"(b) CHILD leaf spans: {len(child_spans)} distinct")
    for f, n in child_spans.most_common(15):
        print(f"    {n:4d}  {f}")
    # (c) occurrence identity of the children, parsed out of the achildren strings
    occ = Counter()
    for r in fam:
        for entry in top_split(field(r, "achildren") or ""):
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
    # O3 must NOT count OccurrenceMinted: EVERY node carries a minted OccurrenceId, so treating
    # Minted as 'synthetic' makes O3 fire for the entire corpus and tells you nothing. Only
    # OccurrenceSynthetic means 'this node has no source position' -- that is the case whose
    # producer is a minter rather than a piece of source.
    sel = []
    minted = sum(n for k, n in occ.items() if "Synthetic" in k)
    kern = sum(n for f, n in files.items() if f.startswith("<kernel"))
    if kern == len(fam) and len(child_spans) and all("kernel" in c for c in child_spans):
        sel.append("O1: every leaf points at a kernel location -> builtin-generic substrate fact; std prerequisite.")
    elif len(child_spans) <= 10:
        sel.append("O2: leaves cluster on %d source position(s) -> producer is a source construct." % len(child_spans))
    elif len(child_spans) > 10:
        sel.append("O4: %d distinct leaf positions -> no single producer; binding-policy question." % len(child_spans))
    if occ and minted >= sum(occ.values()):
        sel.append("O3: every child is OccurrenceSynthetic -> name the minting routine from decl/expr columns.")
    emptyish = sum(1 for r in fam if not (field(r, "bdecl") or "").strip("{}\"' "))
    if emptyish == len(fam):
        sel.append("O5: declaration column uniformly empty -> probe did not measure; do not conclude.")
    for line in sel:
        print("  " + line)
    # One deliverable, not a menu. If the data selects none or more than one, the probe failed to
    # answer the question and must be fixed rather than reported around.
    assert len(sel) == 1, ("expected exactly one pre-registered outcome, got %d: %s" % (len(sel), sel))
    print("\nSELECTED OUTCOME: " + sel[0])
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
