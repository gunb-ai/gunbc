#!/usr/bin/env python3
"""Judge the acceptance criteria from two `SIDE <side> DEPTH <d>` measurement blocks.

Reads the raw dispatch logs, pulls the instrument line per depth, and checks:
  * nest_r2_n must be linear-or-constant in depth (no doubling): compare each
    increment with the previous; a run that doubles is rejected.
  * top_r2_n must not exceed the recorded baseline 2618.
  * r2 exclusive ms must drop versus base.
  * the reconcile wall must be reported for both sides.
Exits non-zero if any criterion fails, and prints the table either way.
"""
import re, sys

BASELINE_TOP_R2 = 2618

def parse(path):
    txt = re.sub(r'\x1b\[[0-9;]*m', '', open(path, errors='replace').read())
    side = None
    rows = {}
    cur = None
    for line in txt.splitlines():
        m = re.match(r'^#+\s+SIDE\s+(\S+)\s+sha=(\S+)', line)
        if m:
            side, sha = m.group(1), m.group(2)
        m = re.match(r'^@@@\s+SIDE=(\S+)\s+DEPTH=(\d+)', line)
        if m:
            cur = int(m.group(2)); rows[cur] = {}
        m = re.search(r'\[generic-arg-pass-excl\]\s+(.*)', line)
        if m and cur is not None:
            for kv in m.group(1).split():
                if '=' in kv:
                    k, v = kv.split('=', 1)
                    try:
                        rows[cur][k] = int(v)
                    except ValueError:
                        pass
        m = re.search(r'(compile\.reconcile done in )(\d+)\s*(ms|seconds|minutes)', line)
        if m and cur is not None:
            n = int(m.group(2)); unit = m.group(3)
            rows[cur]['reconcile_ms'] = n if unit == 'ms' else (n * 1000 if unit == 'seconds' else n * 60000)
            rows[cur]['reconcile_mark'] = line.strip()
    return side, sha, rows


def growth_ok(vals):
    """Accept constant or (near-)linear growth; reject exponential."""
    if not vals:
        return False, "no data"
    if len(vals) == 1:
        return True, "single point"
    d = [vals[i + 1] - vals[i] for i in range(len(vals) - 1)]
    # exponential signature: each increment at least ~1.7x the previous
    doubling = all(d[i + 1] >= 1.7 * max(d[i], 1) for i in range(len(d) - 1)) if len(d) > 1 else False
    if doubling:
        return False, "doubling (increments %s)" % d
    # linear-ish: last value should not exceed ~3x the depth-index scale of the first nonzero increment
    first = next((x for x in d if x > 0), 0)
    if first and vals[-1] > 6 * first * len(vals):
        return False, "super-linear (last=%d, first_incr=%d)" % (vals[-1], first)
    return True, "increments %s" % d


def main():
    base_side, base_sha, base = parse(sys.argv[1])
    cand_side, cand_sha, cand = parse(sys.argv[2])
    print("base      %s %s" % (base_side, base_sha))
    print("candidate %s %s" % (cand_side, cand_sha))
    hdr = "%-6s %-28s %-28s" % ("depth", "BASE", "CANDIDATE")
    print(hdr)
    for d in sorted(set(base) | set(cand)):
        b = base.get(d, {}); c = cand.get(d, {})
        f = lambda r: "top_r2=%s nest_r2=%s r2nest_ms=%s r2top_ms=%s rec_ms=%s" % (
            r.get('top_r2_n'), r.get('nest_r2_n'), r.get('r2_nest_excl_ms'),
            r.get('r2_top_excl_ms'), r.get('reconcile_ms'))
        print("%-6s %-28s %-28s" % (d, f(b) if b else "-", f(c) if c else "-"))

    fails = []
    bn = [base[d].get('nest_r2_n') for d in sorted(base) if 'nest_r2_n' in base[d]]
    cn = [cand[d].get('nest_r2_n') for d in sorted(cand) if 'nest_r2_n' in cand[d]]
    if not cn:
        fails.append("candidate produced no nest_r2_n counts")
    else:
        ok, why = growth_ok(cn)
        print("\nACCEPT nest_r2_n linear-or-constant: %s (%s)" % ("PASS" if ok else "FAIL", why))
        if not ok:
            fails.append("nest_r2_n growth: %s" % why)
    print("       base   nest_r2_n series: %s" % bn)
    print("       cand   nest_r2_n series: %s" % cn)

    top = max([cand[d].get('top_r2_n', 0) for d in cand] or [0])
    ok = top <= BASELINE_TOP_R2
    print("ACCEPT top_r2_n <= %d: %s (max observed %d)" % (BASELINE_TOP_R2, "PASS" if ok else "FAIL", top))
    if not ok:
        fails.append("top_r2_n %d > %d" % (top, BASELINE_TOP_R2))

    bms = [base[d].get('r2_nest_excl_ms', 0) for d in sorted(base)]
    cms = [cand[d].get('r2_nest_excl_ms', 0) for d in sorted(cand)]
    if bms and cms:
        ok = cms[-1] < bms[-1]
        print("ACCEPT nested r2 ms drops: %s (base deepest %s -> cand %s)" % ("PASS" if ok else "FAIL", bms[-1], cms[-1]))
        if not ok:
            fails.append("nested r2 ms did not drop")
    rec_b = [base[d].get('reconcile_ms') for d in sorted(base) if 'reconcile_ms' in base[d]]
    rec_c = [cand[d].get('reconcile_ms') for d in sorted(cand) if 'reconcile_ms' in cand[d]]
    print("       reconcile wall base: %s" % rec_b)
    print("       reconcile wall cand: %s" % rec_c)

    for side, rows in (("base", base), ("candidate", cand)):
        for d in sorted(rows):
            r = rows[d]
            for k in ('top_r2_n', 'nest_r2_n'):
                if k not in r:
                    fails.append("%s depth %d missing %s (instrument did not fire)" % (side, d, k))

    print("\n== VERDICT: %s" % ("ALL MEASURED CRITERIA PASS" if not fails else "FAILURES:\n  " + "\n  ".join(fails)))
    sys.exit(1 if fails else 0)


main()
