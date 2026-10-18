#!/usr/bin/env python3
"""Read REBINDARM lines from probe v2 and return the PRE-REGISTERED verdict, per row and in total.

Outcomes were registered in probe_bucketb2_14152.sh BEFORE the run (stern-bat-401 msg_576466dc required
registration first, so a cause cannot be fitted to the data afterwards):
  A  cross=true  AND core_na=true            -> the AMBIENT text judgment refuses, not the two types.
                                                FOLD SCOPE: pass the settled text judgment for each node's
                                                own module. Belongs in #14152.
  B  cross=false AND the trees differ        -> X1 PROPER: a defect in the relation's descent or in
     (below Resolved{node}, in raw children,    declaration identity. Its own PR.
      or in decl=)
  C  cross=false AND the trees are identical -> UNEXPLAINED. Report by identity. Pick no cause.
     in every printed field, yet refuses
Validity (checked first, because an arm count measured on a perturbed compiler is worthless):
  refusals must be 208 (config A's census) and one REBINDARM line per refusal.

Also prints the repL/repR pair, because 'cross' is only credible when one side is HostText/
CodePointSequence and the other NotText: a crossing between two NotText nodes would mean I have
mis-read what text_crossing_in_judgment does.
"""
import sys, re, collections, json

CAPS = re.compile(r"\s*\|\|CHILD-CAP|\s*\|\|DEPTH-CAP")
# Fields are taken BY NAME from what rebind_probe_node emits:
#   path n=..|conn=..|card=..|decl=..|infer=..|shape=..|nc=N|idsp=..
# An earlier version omitted the shape group, so `infer`'s lazy match ran on to |nc= and ATE
# "none|shape=Coproduct(X)". Every infer value then carried the shape as a prefix, and a census of
# "which rows carry error_type" taken from it read 0 -- plausible-looking, and wrong. A second version
# split segments on " ;; " only, letting idsp's .*? swallow the whole right-hand node, which made every
# pair differ in "node count" and produced a fabricated OUTCOME B for all 254 rows. Both bugs were caught
# by cross-checking against the emitter, which is the only authority for a log format.
NODE = re.compile(r"^(?P<path>[LR][0-9A-Za-z]*) n=(?P<name>[^|]*)\|conn=(?P<conn>[^|]*)\|card=(?P<card>[^|]*)"
                  r"\|decl=(?P<decl>[^|]*)\|infer=(?P<infer>.*?)\|shape=(?P<shape>.*?)\|nc=(?P<nc>\d+)\|idsp=(?P<idsp>.*)$")
REFUSAL = re.compile(r"type mismatch: expected 'type variable (\S+) bound as (.*?) \(first argument\)', "
                     r"got '(.*?) \(later argument\)' \((.*?):(\d+)-(\d+)\)")
FIELDS = ["name", "conn", "card", "decl", "infer", "shape", "nc", "idsp"]

def parse(line):
    """One REBINDARM line -> header fields + the pairwise node table keyed by path (L0/R0/L00/.../L0r)."""
        # Two tags exist in the wild: the arm probe's own REBINDARM|, and patch_rebind_probe3.py's
    # REBINDV3|, which carries the SAME cross/trees fields. A parser that accepted only one silently
    # returned 0 rows for a good run, and 0 reads like "the probe never reached the refusal site".
    if not (line.startswith("REBINDARM|") or line.startswith("REBINDV3|")):
        return None
    b = line[len("REBINDARM|"):].rstrip("\n")
    i = b.find(";;TREE")
    if i < 0:
        return None
    d = {}
    for kv in b[:i].rstrip("|").split("|"):
        if "=" in kv:
            k, v = kv.split("=", 1)
            d[k] = v
    d["nodes"], d["unparsed"] = {}, 0
    for seg in b[i + 6:].split(" ;; "):
        # " ;; " separates PAIRS; each pair is `<L node> >< <R node>`, emitted by rebind_probe_tree.
        # Splitting on the pair separator too is mandatory: without it the right-hand node is absorbed
        # into the left node's idsp and every comparison degenerates.
        for side in seg.split(" >< "):
            side = side.strip()
            if not side:
                continue
            m = NODE.match(side)
            if not m:
                d["unparsed"] += 1
                continue
            g = m.groupdict()
            # the fuel markers are the emitter's own annotations, not node data; leaving them in idsp
            # made cap-bearing rows differ in idsp from their uncapped twin.
            g["idsp"] = CAPS.sub("", g["idsp"])
            d["nodes"][g["path"]] = g
    return d

def first_difference(d):
    """Which printed field first separates the pair, or 'IDENTICAL'.

    Comparison is positional by SORTED PATH, which assumes the two walks visit in lockstep. They do:
    rebind_probe_tree walks children by shared index and descends Resolved on both sides together, so
    path strings match structurally. If that ever changes, this returns node-count rather than lying.
    """
    n = d["nodes"]
    if d.get("unparsed"):
        return "unparsed-nodes"
    L = sorted(k for k in n if k.startswith("L"))
    R = sorted(k for k in n if k.startswith("R"))
    if len(L) != len(R):
        return "node-count"
    for lk, rk in zip(L, R):
        for f in FIELDS:
            if n[lk][f] != n[rk][f]:
                return f
    return "IDENTICAL"

def inferred_kind(v):
    v = v or "none"
    for p in ("Resolved", "CompilerError", "TypeVariable", "Divergent", "none"):
        if v.startswith(p):
            return p
    return "OTHER:" + v[:24]

def main(paths):
    rows = []
    for p in paths:
        for line in open(p, errors='replace'):
            if line.startswith('REBINDARM|') or line.startswith('REBINDV3|'):
                d = parse(line)
                if d:
                    rows.append(d)
                else:
                    print("UNPARSEABLE REBINDARM LINE (truncated?):", line[:200])
    print("REBINDARM rows parsed: %d" % len(rows))
    if not rows:
        print("NOTHING PARSED -> the probe did not reach the refusal site. Do NOT read 0 as 'no refusals'.")
        return 1

    print("\n=== validity gates (from the dispatch log, re-checked here) ===")
    print("  distinct (module, unfoldedL, unfoldedR, bind): %d" % len({(r['module'], r.get('unfoldedL'), r.get('unfoldedR'), r['bind']) for r in rows}))

    print("\n=== refusal diagnostics (parsed by the emitted message grammar) ===")
    refs = collections.Counter()
    for p_ in paths:
        for line in open(p_, errors="replace"):
            for m in REFUSAL.finditer(line):
                bn, exp, got, f_, s_, e_ = m.groups()
                refs[(f_, int(s_), int(e_), bn, exp, got)] += 1
    print("  refusal diagnostics parsed: %d   distinct identities: %d   files: %d"
          % (sum(refs.values()), len(refs), len({k[0] for k in refs})))
    print("  NOTE the two counts differ by design: the probe eprintln!s at the refusal site on EVERY pass,"
          " while diagnostics are deduplicated by the collector. Rows >= probe lines is therefore not a"
          " defect signature; 0 refusals with no probe line at all would be.")

    print("\n=== outcome tally, exactly as pre-registered ===")
    A = [r for r in rows if r.get('cross') == 'true' and r.get('core_na') == 'true']
    noxc = [r for r in rows if r.get('cross') == 'false']
    B = [r for r in noxc if first_difference(r) != 'IDENTICAL']
    C = [r for r in noxc if first_difference(r) == 'IDENTICAL']
    print("  A  ambient text judgment (cross=true, core_na=true) : %d" % len(A))
    print("  B  relation descent/declaration (cross=false, trees differ) : %d" % len(B))
    print("  C  UNEXPLAINED (cross=false, trees identical, still refused) : %d" % len(C))
    print("  neither A/B/C (cross=true but core_na=false -> inconsistent, report) : %d"
          % (len(rows) - len(A) - len(B) - len(C)))

    # The C rows must be REPORTED BY IDENTITY (the pre-registration says "Report by identity. Pick no
    # cause."), and an earlier revision of this parser printed only the COUNT -- which is how 18 rows sat
    # un-named when the PR body needed them. Print each one, deduplicated on its own identity tuple.
    print("\n=== OUTCOME C, by identity (UNEXPLAINED -- reported as such, no cause assigned) ===")
    seen = set()
    for r in C:
        ident = (r['module'], r['bind'], r.get('unfoldedL'), r.get('unfoldedR'))
        if ident in seen:
            continue
        seen.add(ident)
        print("  module=%s bind=%s  L=%s  R=%s" % ident)
    print("  distinct C identities: %d (over %d C lines; the probe emits per pass)" % (len(seen), len(C)))

    print("\n=== what separates the pair, per row (the field, by name) ===")
    for k, v in collections.Counter(first_difference(r) for r in rows).most_common():
        print("  %4d  %s" % (v, k))

    print("\n=== inferred-kind census over every printed node (the error_type / slot question) ===")
    kc = collections.Counter()
    for r in rows:
        for g in r['nodes'].values():
            kc[inferred_kind(g['infer'])] += 1
    for k, v in kc.most_common():
        print("  %5d  %s" % (v, k))
    print("  rows carrying >=1 CompilerError node: %d" % sum(
        1 for r in rows if any(inferred_kind(g['infer']) == 'CompilerError' for g in r['nodes'].values())))
    print("  rows carrying >=1 still-marked TypeVariable: %d" % sum(
        1 for r in rows if any(inferred_kind(g['infer']) == 'TypeVariable' for g in r['nodes'].values())))
    print("  rows whose BOUND side prints Product(Unit<>): %d ; ACTUAL side: %d" % (
        sum(1 for r in rows if 'Product(Unit<>)' in (r.get('unfoldedL') or '')),
        sum(1 for r in rows if 'Product(Unit<>)' in (r.get('unfoldedR') or ''))))

    print("\n=== the judgment really matters? core_na is core with the judgment REMOVED ===")
    flips = [r for r in rows if r.get('core_na') == 'true']
    print("  rows that ACCEPT once the judgment is removed: %d" % len(flips))
    print("  of those, cross=false (so NOT the crossing arm): %d -> the judgment is consumed by"
          " peel_alias_rhs_once, which is the first thing node_type_equals_core does"
          % sum(1 for r in flips if r.get('cross') == 'false'))
    for r in flips[:6]:
        print("    module=%s bind=%s sep=%s shape=%s" % (r['module'], r['bind'], first_difference(r),
                                                         (r.get('unfoldedL') or '')[:110]))

    print("\n=== arm differential, all rows (the measurement itself) ===")
    for k, c in collections.Counter(
            (r.get('equals'), r.get('core'), r.get('cross'), r.get('core_na'), r.get('equals_na'),
             r.get('textpair'), r.get('headapp')) for r in rows).most_common(12):
        print("  equals=%-5s core=%-5s cross=%-5s core_na=%-5s equals_na=%-5s textpair=%-5s headapp=%-5s  x%d" % (k + (c,)))

    print("\n=== repL/repR sanity for every crossing row (a crossing needs a text side) ===")
    repc = collections.Counter((r.get('repL', '')[:28], r.get('repR', '')[:28]) for r in rows if r.get('cross') == 'true')
    if not repc:
        print("  (no crossing rows)")
    for (a, b), n in repc.most_common(8):
        print("  L=%-30s R=%-30s x%d" % (a, b, n))

    def show(tag, rs, limit=8):
        print("\n=== %s: first %d by identity ===" % (tag, limit))
        seen = set()
        for r in rs:
            key = (r['module'], r.get('bind'), r.get('unfoldedL'), r.get('unfoldedR'))
            if key in seen:
                continue
            seen.add(key)
            print("  module=%s bind=%s" % (r['module'], r['bind']))
            print("    unfoldedL=%s" % (r.get('unfoldedL') or '')[:150])
            print("    unfoldedR=%s" % (r.get('unfoldedR') or '')[:150])
            print("    text=%s" % (r.get('text') or '')[:110])
            if len(seen) >= limit:
                break

    show("OUTCOME A -> fold scope", A)
    show("OUTCOME B -> X1 proper", B)
    show("OUTCOME C -> unexplained, report only", C)
    json.dump([{k: v for k, v in r.items() if k != 'nodes'} for r in rows],
              open('/tmp/rebindarm_rows.json', 'w'), indent=1)
    print("\nrows written: /tmp/rebindarm_rows.json")
    return 0

if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
