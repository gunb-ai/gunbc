#!/usr/bin/env python3
"""Summarize a shadow-arg-conformance sidecar TSV. Fail-closed: every row is counted in
exactly one outcome bucket, and an unknown outcome tag is a refusal, not a residue."""
import sys, csv, collections

path = sys.argv[1]
rows = list(csv.DictReader(open(path), delimiter='\t'))
KNOWN = {"Compatible","WouldDiagnose","ComparisonUnavailable","RepresentationRelationUnadjudicated"}
bad = [r for r in rows if r["outcome"] not in KNOWN]
if bad:
    print("REFUSE: unknown outcome tags:", {r["outcome"] for r in bad}); sys.exit(9)

print(f"total relation rows: {len(rows)}")
for scope in ("exempt","judged"):
    sub=[r for r in rows if r["exempt"]==scope]
    c=collections.Counter(r["outcome"] for r in sub)
    print(f"\n[{scope}]  rows={len(sub)}")
    for k in ("Compatible","WouldDiagnose","ComparisonUnavailable","RepresentationRelationUnadjudicated"):
        print(f"   {k:38s} {c.get(k,0)}")
    cz=collections.Counter(r["cause"] for r in sub if r["cause"])
    for k,v in cz.most_common():
        print(f"      cause {k:44s} {v}")

wd=[r for r in rows if r["outcome"]=="WouldDiagnose" and r["exempt"]=="exempt"]
print(f"\n=== EXEMPT WouldDiagnose (candidate source conformance debt): {len(wd)} ===")
print("\nby caller module:")
for m,n in collections.Counter(r["caller_module"] for r in wd).most_common():
    print(f"  {n:5d}  {m}")
print("\nby (formal_type -> actual_type) pair, top 25:")
for p,n in collections.Counter(f'{r["formal_type"]}  <-  {r["actual_type"]}' for r in wd).most_common(25):
    print(f"  {n:5d}  {p}")
print("\ndistinct call sites (module|decl|callee|formal):",
      len({(r["caller_module"],r["caller_decl"],r["callee"],r["formal_index"]) for r in wd}))
