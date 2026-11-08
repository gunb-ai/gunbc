#!/usr/bin/env python3
"""Verify the #16061 measurement receipt bundle (node adhoc-afb0f800-78c).

Reads: verdicts.txt, outcomes.json (ledger TSV), bin.sha256.before/after,
END_receipt.txt (OUTCOME line), plus the packet's own FUNCTIONS roster.
Prints a per-cell table and prints NO_OBSERVATION with the reason if any
integrity rule fails. RED CELLS ARE DATA: a FAIL is a result, not a failure.
Usage: verify_nal.py <dir> [--roster csv]
"""
import sys, os, re, csv

CAT = lambda n: ('ARM' if '_arm_' in n else
                 'CONTROL' if any(k in n for k in ('structurally_equal', 'accept', '_refuses_')) else
                 'NODE')

def die(reason):
    print("NO_OBSERVATION:", reason)
    sys.exit(2)

def main(d, roster_csv=None):
    def rd(name, required=True):
        p = os.path.join(d, name)
        if not os.path.exists(p):
            if required: die("missing artifact %s" % name)
            return None
        return open(p).read()

    verdicts_raw = rd('verdicts.txt')
    lines = [l.rstrip('\n') for l in verdicts_raw.splitlines() if l.strip()]
    bad = [l for l in lines if not re.fullmatch(r'(PASS|FAIL) nal_[A-Za-z0-9_]+', l)]
    if bad:
        die("verdict line is not bare 'PASS|FAIL <name>' (a suffix means the cell "
            "was not attempted / errored / returned non-Bool): %r" % bad[:5])

    got = {}
    for l in lines:
        v, n = l.split(' ', 1)
        if n in got: die("duplicate verdict line for %s" % n)
        got[n] = v

    roster = [x.strip() for x in roster_csv.split(',')] if roster_csv else sorted(got)
    missing = [n for n in roster if n not in got]
    extra = [n for n in got if n not in roster]
    if missing: die("roster cell has no verdict line: %s" % missing[:5])
    if extra:   die("verdict line not in the roster: %s" % extra[:5])

    outcome = rd('END_receipt.txt', required=False)
    if outcome:
        m = re.search(r'OUTCOME=(\S+)\s+rc=(\d+)', outcome)
        if not m: die("END_receipt has no OUTCOME line")
        if m.group(1) != 'OBSERVED' or m.group(2) != '0':
            die("OUTCOME=%s rc=%s" % (m.group(1), m.group(2)))
        n_m = re.search(r'table_complete\s+(\d+)/(\d+)', outcome)
        if n_m and n_m.group(1) != n_m.group(2):
            die("table_complete %s/%s is not complete" % (n_m.group(1), n_m.group(2)))

    b0, b1 = rd('bin.sha256.before'), rd('bin.sha256.after')
    if b0 and b1 and b0.split()[0] != b1.split()[0]:
        die("binary changed between P1 and P3: %s vs %s" % (b0.split()[0][:12], b1.split()[0][:12]))

    ledger = {}
    detail = {}
    p = os.path.join(d, 'outcomes.json')
    if os.path.exists(p):
        for row in csv.reader(open(p), delimiter='\t'):
            if len(row) >= 3 and ('.' in row[0] or row[0].startswith('gunbc')):
                key = row[0].split('.')[-1]
                ledger.setdefault(key, []).append((row[1], row[2]))
                detail[key] = row[3] if len(row) > 3 else ''

    # ledger cross-check, mirroring the script's own rule independently: exactly one row per roster
    # cell, expect in {expect-hold, expect-red}, returned agrees with the bare verdict line, no extras.
    if os.path.exists(p):
        bad = []
        for n, v in got.items():
            rows = ledger.get(n)
            if rows is None:
                bad.append("%s(no_ledger_row)" % n); continue
            if len(rows) != 1:
                bad.append("%s(ledger_rows:%d)" % (n, len(rows))); continue
            expect, returned = rows[0]
            if expect not in ('expect-hold', 'expect-red'):
                bad.append("%s(expect:%s)" % (n, expect))
            want = 'returned-true' if v == 'PASS' else 'returned-false'
            if returned != want:
                bad.append("%s(ledger:%s verdict:%s)" % (n, returned, want))
        extra = [k for k in ledger if k not in got]
        if extra:
            bad.append("extra_ledger_rows:%s" % ",".join(sorted(extra)[:5]))
        if bad:
            die("ledger_disagrees_with_verdicts:%s" % "; ".join(bad[:8]))

    print("cells=%d  PASS=%d  FAIL=%d" % (len(got), sum(v == 'PASS' for v in got.values()),
                                          sum(v == 'FAIL' for v in got.values())))
    from collections import defaultdict
    by = defaultdict(list)
    for n, v in sorted(got.items()): by[CAT(n)].append((n, v))
    for c in ('CONTROL', 'ARM', 'NODE'):
        rows = by.get(c, [])
        print("\n%s (%d cells, %d pass / %d fail)" % (c, len(rows),
              sum(v == 'PASS' for _, v in rows), sum(v == 'FAIL' for _, v in rows)))
        for n, v in rows:
            rows = ledger.get(n) or [('-', '-')]
            print("   %-4s %-56s %-16s %s" % (v, n, '%s/%s' % rows[0], (detail.get(n) or '')[:60]))
    print("\nOBSERVED")

if __name__ == '__main__':
    d = sys.argv[1]
    r = None
    if '--roster' in sys.argv: r = sys.argv[sys.argv.index('--roster') + 1]
    main(d, r)
