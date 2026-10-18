#!/usr/bin/env python3
"""Analyse REBINDV9| probe rows: does widening the predicate to generic_names fire on the Map family?

WHY THIS FILE EXISTS RATHER THAN AN INLINE python -c:
  An inline field extractor here used r'\\|ghit=\\([^|]*\\)'. In a regex, \\( is a LITERAL paren, so that
  pattern matches nothing, and the helper returned '' for every field -- which is indistinguishable from
  "the probe measured zero". That produced a confident, wrong conclusion (ghit=0/235, "widening cannot
  fire") that was only caught by noticing a contradiction: 12 rows printed ghit=1 while their own
  bparams/aparams printed empty, and ownBound=0 on every row contradicted the v4 instrument's 141.
  So this script asserts extraction health BEFORE it reports anything: if a field that must be present
  comes back None, it stops rather than reporting zeros. A silent empty is the failure mode.

Reads a file of REBINDV9| rows (one per line) as argv[1].
"""
import re
import sys
from collections import Counter

FIELDS = ("module", "bind", "formal", "gnames", "ghit", "ghitact",
          "ownBound", "ownActual", "bparams", "aparams", "bleaves", "aleaves",
          "lconn", "rconn")


def field(row, key):
    """Fetch one field. UNESCAPED capturing parens -- `\\(` would be a literal and silently miss."""
    m = re.search(r"\|" + re.escape(key) + r"=([^|]*)", row)
    if m is None:
        return None            # absent is NOT the same as empty; callers must not collapse them
    return m.group(1)


def leaves(row, key):
    v = field(row, key)
    if v in (None, ""):
        return set()
    return {x for x in v.split(",") if x}


def main(path):
    rows = [l for l in open(path, encoding="utf-8", errors="replace").read().split("\n")
            if l.startswith("REBINDV9|")]
    if not rows:
        print("NO ROWS: the probe emitted nothing. This is not a measurement of zero.")
        return 1

    # EXTRACTION HEALTH CHECK. ghit/ownBound/bleaves are always emitted by the probe, so None here means the
    # extractor is broken, and every count below would be a false zero.
    for k in ("ghit", "ghitact", "ownBound", "ownActual"):
        if any(field(r, k) is None for r in rows):
            print(f"EXTRACTOR BROKEN: field '{k}' absent from at least one row. "
                  f"Refusing to report counts. (A silent empty reads exactly like a measured zero.)")
            return 2
    print(f"rows={len(rows)}  extractor healthy (ghit/ownBound present on every row)")

    for k in ("ghit", "ghitact", "ownBound", "ownActual"):
        print(f"  {k:9s} {dict(Counter(field(r, k) for r in rows))}")

    # THE family the disposition turns on: bound side is a Map whose leaves are the builtin Map's own
    # parameters K and V (the shape CI reported: 'type variable A bound as Map(Primitive(K),Primitive(V))').
    mapfam = [r for r in rows
              if leaves(r, "bleaves") and leaves(r, "bleaves") <= {"K", "V"}]
    print(f"\nMAP FAMILY (bound-side childless leaves ⊆ {{K,V}}): {len(mapfam)} rows")
    if mapfam:
        for k in ("ghit", "ghitact", "ownBound", "ownActual"):
            print(f"  {k:9s} {dict(Counter(field(r, k) for r in mapfam))}")
        print("  by identity (module, bind, bleaves, gnames):")
        tab = Counter((field(r, "module"), field(r, "bind"),
                       ",".join(sorted(leaves(r, "bleaves"))), field(r, "gnames"))
                      for r in mapfam)
        for k, n in tab.most_common(12):
            print(f"    {n:4d}  {k[0]} bind={k[1]} bleaves={k[2]} gnames={k[3]}")
        print("\n  THE DECISION: ghit on this family is what says whether comparing the bound side's")
        print("  childless leaves against generic_names (instead of gating on n.params) would fire.")
        gh = sum(1 for r in mapfam if field(r, "ghit") == "1")
        if len(mapfam) == 0:
            print("    -> NO map-family rows in this corpus: the question is UNANSWERED here, not answered 'no'.")
        elif gh == 0:
            print(f"    -> ghit=0 on all {len(mapfam)}: widening would NOT fire on this family.")
        else:
            print(f"    -> ghit=1 on {gh}/{len(mapfam)}: widening WOULD fire; build the predicate.")

    # Consistency guard: ownBound must equal 'bparams non-empty AND some bleaf names a param'.
    # If it does not, the probe's own arithmetic disagrees with its printed inputs and the rows cannot be
    # trusted -- that is exactly the contradiction that exposed the earlier helper bug.
    bad = 0
    for r in rows:
        bp, bl = leaves(r, "bparams"), leaves(r, "bleaves")
        expect = "1" if (bp and (bp & bl)) else "0"
        if expect == "1" and field(r, "ownBound") != "1":
            bad += 1
    print(f"\nCONSISTENCY: rows where inputs imply ownBound=1 but the field says otherwise: {bad}")
    if bad:
        print("  -> the probe's helper disagrees with its own printed inputs. Do NOT report these numbers;")
        print("     re-check the helper against src/v1/04_infer.dag's node_mentions_own_parameter first.")
    # and the widened analogue
    badg = 0
    for r in rows:
        gn, bl = set((field(r, "gnames") or "").split(",")) - {""}, leaves(r, "bleaves")
        expect = "1" if (gn and (gn & bl)) else "0"
        if expect == "1" and field(r, "ghit") != "1":
            badg += 1
    print(f"CONSISTENCY (widened): inputs imply ghit=1 but field says otherwise: {badg}")

    # identity rollup, for the PR body's by-identity tables
    print(f"\nDISTINCT identities (module, bind): {len({(field(r,'module'), field(r,'bind')) for r in rows})}")
    print("ghit=1 identities:")
    for t, n in Counter((field(r, "module"), field(r, "bind"), field(r, "bleaves")[:40])
                        for r in rows if field(r, "ghit") == "1").most_common(15):
        print(f"    {n:4d}  {t[0]} bind={t[1]} bleaves={t[2]}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
