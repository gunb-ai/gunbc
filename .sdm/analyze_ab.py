#!/usr/bin/env python3
"""Judge the #14075 A/B from one dispatch log.

Parses `@@@@ VARIANT=... / @@@ VARIANT=... SERIES=... DEPTH=...` blocks emitted by
measure_variant.sh and reports, per variant and series, the parent's three columns:

  top-level infer work  r1_pass_n_top      first-round argument folds entered at top level
  necessary rechecks    recheck_n_top      re-derivations entered with none other active
  nested recheck work   recheck_n_nest     re-derivations entered while another was open
                                           == work done MORE THAN ONCE (the cascade)

Bar (parent, revised acceptance):
  * left-bound (gap_depth): necessary rechecks == 0 at every depth, per-level increment
    of nested work ~constant, i.e. neither doubling NOR quadratic.
  * right-bound (garr): rechecks genuinely required; must stay polynomial, with the
    per-level increment explained (quadratic is admissible here only if it is THE
    necessary work, and we say so).
  * control-vs-treatment on the same tree is the comparison; the historical 3ead368
    numbers are context only.
Exits non-zero if any bar fails.
"""
import re
import sys

LINE = re.compile(r"^\[(\d{2}:\d{2}:\d{2})\d*.*$")


def parse(path):
    txt = open(path, errors="replace").read()
    txt = re.sub(r"\x1b\[[0-9;]*m", "", txt)
    variants = {}
    cur = None
    variant = None
    for line in txt.splitlines():
        m = re.match(r"^@@@@ VARIANT=(\S+)", line)
        if m:
            variant = m.group(1)
            variants.setdefault(variant, {})
            continue
        m = re.match(r"^@@@ VARIANT=(\S+) SERIES=(\S+) DEPTH=(\d+)", line)
        if m:
            variant = m.group(1)
            cur = (m.group(2), int(m.group(3)))
            variants.setdefault(variant, {})[cur] = {}
            continue
        m = re.match(r"^@@@ VARIANT=(\S+) SPECIMEN=(\S+)", line)
        if m:
            variant = m.group(1)
            cur = ("specimen", 0)
            variants.setdefault(variant, {})[cur] = {}
            continue
        m = re.match(r"^@@@ VARIANT=(\S+) WITNESS\b", line)
        if m:
            variant = m.group(1)
            cur = None
            continue
        m = re.search(r"\[generic-arg-pass-excl\]\s+(.*)", line)
        if m and cur is not None and variant is not None:
            for kv in m.group(1).split():
                if "=" in kv:
                    k, v = kv.split("=", 1)
                    try:
                        variants[variant][cur][k] = int(v)
                    except ValueError:
                        pass
            continue
        m = re.search(r"reconcile: .*?done in (\d+)(ms|seconds|minutes)", line)
        if m and cur is not None and variant is not None:
            n = int(m.group(1))
            mult = {"ms": 1, "seconds": 1000, "minutes": 60000}[m.group(2)]
            variants[variant][cur]["reconcile_ms"] = n * mult
            continue
        m = re.match(r"^duplicate_diagnostics=(\d+)", line)
        if m and cur is not None and variant is not None:
            variants[variant][cur]["dup_diag"] = int(m.group(1))
            continue
        m = re.match(r"^witness_rc=(\d+)", line)
        if m and variant is not None:
            variants[variant].setdefault(("witness", 0), {})["rc"] = int(m.group(1))
            continue
        m = re.match(r"^typemismatch_mentions=(\d+)", line)
        if m and variant is not None:
            variants[variant].setdefault(("witness", 0), {})["typemismatch"] = int(m.group(1))
    return variants


COLS = [("r1_pass_n_top", "top_infer"), ("recheck_n_top", "nec_rchk"),
        ("recheck_n_nest", "nest_rchk"), ("recheck_nest_excl_ms", "nest_ms"),
        ("reconcile_ms", "reconcile_ms")]


def table(name, rows):
    print("\n== %s ==" % name)
    print("%-6s %s" % ("depth", " ".join("%-13s" % c[1] for c in COLS)))
    for d in sorted(d for (s, d) in rows if s == name):
        r = rows[(name, d)]
        print("%-6s %s" % (d, " ".join("%-13s" % r.get(k, "-") for k, _ in COLS)))


def series(rows, s):
    return [rows[(s, d)].get(k) for (ss, d) in sorted(rows) if ss == s for k in ["x"]][:0] or \
           [rows[(s, d)] for (ss, d) in sorted(rows) if ss == s]


def classify(vals, label):
    """Return (verdict, reason). Reject doubling; report growth shape."""
    vals = [v for v in vals if v is not None]
    if len(vals) < 3:
        return "INSUFFICIENT", "only %d points" % len(vals)
    d1 = [vals[i + 1] - vals[i] for i in range(len(vals) - 1)]
    d2 = [d1[i + 1] - d1[i] for i in range(len(d1) - 1)]
    ratios = [vals[i + 1] / max(vals[i], 1) for i in range(len(vals) - 1) if vals[i] > 0]
    if any(r >= 1.6 for r in ratios) and len(ratios) >= 2 and all(r >= 1.5 for r in ratios):
        return "EXPONENTIAL", "ratios %s" % [round(r, 2) for r in ratios]
    # quadratic: second differences grow ~linearly while staying positive
    if all(x >= 0 for x in d2) and len(d2) >= 2 and d2[-1] > 2 * max(1, d2[0]):
        return "QUADRATIC", "d1=%s d2=%s" % (d1, d2)
    if all(-1 <= x <= 1 for x in d1):
        return "CONSTANT", "d1=%s" % d1
    return "LINEAR", "d1=%s" % d1


def main():
    path = sys.argv[1]
    vs = parse(path)
    if not vs:
        print("NO VARIANTS PARSED in %s" % path)
        sys.exit(2)
    print("variants seen: %s" % sorted(vs))
    fails = []
    for v in sorted(vs):
        rows = vs[v]
        table("gap_depth (left-bound, recheck NOT required)", rows)
        table("garr (right-bound, recheck REQUIRED)", rows)
        if ("specimen", 0) in rows:
            print("specimen: %s" % rows[("specimen", 0)])
        if ("witness", 0) in rows:
            print("witness: %s" % rows[("witness", 0)])

        def col(s, k):
            return [rows[(s, d)].get(k) for (ss, d) in sorted(rows) if ss == s]

        lb_rchk = col("gap_depth", "recheck_n_top")
        lb_nest = col("gap_depth", "recheck_n_nest")
        rb_nest = col("garr", "recheck_n_nest")
        rb_rchk = col("garr", "recheck_n_top")
        if lb_rchk and any(x not in (0, None) for x in lb_rchk):
            fails.append("%s left-bound NECESSARY rechecks not all zero: %s" % (v, lb_rchk))
        if lb_nest:
            verdict, why = classify(lb_nest, "left-bound nested")
            print("%s left-bound nested-work shape: %s (%s)" % (v, verdict, why))
            if verdict in ("EXPONENTIAL", "QUADRATIC", "INSUFFICIENT"):
                fails.append("%s left-bound nested work %s: %s" % (v, verdict, why))
        if rb_nest:
            verdict, why = classify(rb_nest, "right-bound nested")
            print("%s right-bound nested-work shape: %s (%s)" % (v, verdict, why))
            if verdict == "EXPONENTIAL":
                fails.append("%s right-bound nested work EXPONENTIAL: %s" % (v, why))
        for (s, d), r in rows.items():
            if s == "gap_depth" and r.get("dup_diag"):
                fails.append("%s left-bound d%d duplicate diagnostics %d" % (v, d, r["dup_diag"]))
    print("\n== VERDICT: %s" % ("PASS" if not fails else "FAILURES:\n  " + "\n  ".join(fails)))
    sys.exit(1 if fails else 0)


main()
