#!/usr/bin/env python3
"""Adjudicate every WouldDiagnose row against the corpus's own transparent type aliases.

A row is a REPRESENTATION GAP (not a source defect) exactly when the formal's and the
actual's type names reduce to the SAME base under the corpus's `type A = B` declarations.
The alias table is read from source, so this is decidable, not a judgement call. A row that
does not reduce is printed in full and adjudicated by hand -- it is never absorbed.
"""
import sys, csv, re, subprocess, collections

shadow_path = sys.argv[1]

# Transparent aliases: `type A = B` where B is a bare name (no braces, no type arguments).
alias = {}
out = subprocess.run(
    ["grep","-rhnE",r"^type [A-Za-z_][A-Za-z0-9_]* = [A-Za-z_][A-Za-z0-9_]*$","src/v2","dag","src/v1"],
    capture_output=True, text=True).stdout
for line in out.splitlines():
    m = re.search(r"^\d+:type ([A-Za-z_][A-Za-z0-9_]*) = ([A-Za-z_][A-Za-z0-9_]*)$", line)
    if m:
        alias.setdefault(m.group(1), set()).add(m.group(2))

def base(name, seen=None):
    seen = seen or set()
    if name in seen or name not in alias:
        return name
    seen.add(name)
    tgts = alias[name]
    # A name aliased two different ways is NOT reduced -- ambiguity is not equivalence.
    if len(tgts) != 1:
        return name
    return base(next(iter(tgts)), seen)

def leaf(shape):
    m = re.fullmatch(r"(?:Product|Primitive)\((.*)\)", shape)
    return m.group(1) if m else shape

rows = [r for r in csv.DictReader(open(shadow_path), delimiter='\t')
        if r["outcome"]=="WouldDiagnose" and r["exempt"]=="exempt"]

gap, residue = [], []
for r in rows:
    f, a = leaf(r["formal_type"]), leaf(r["actual_type"])
    (gap if base(f)==base(a) else residue).append((r,f,a,base(f),base(a)))

print(f"alias declarations read from source: {len(alias)}")
print(f"WouldDiagnose relations:            {len(rows)}")
print(f"  reduce to one base (REPRESENTATION GAP, not source debt): {len(gap)}")
print(f"  do NOT reduce (residue, adjudicate by hand):              {len(residue)}")
print()
print("gap population by reduced base:")
for b,n in collections.Counter(g[3] for g in gap).most_common():
    print(f"  {n:5d}  {b}")
if residue:
    print("\nRESIDUE -- every row printed, nothing absorbed:")
    for r,f,a,bf,ba in residue:
        print(f"  {r['caller_module']}.{r['caller_decl']} -> {r['callee']} f{r['formal_index']} "
              f"({r['formal_label']}): {f}[{bf}] <- {a}[{ba}]")
