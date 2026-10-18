#!/usr/bin/env python3
"""B minus A, BY IDENTITY -- what the two 'declaration's own form' arms actually admitted.

Each ab_extension_14152.sh dispatch base64-tars its own artifacts into its dispatch log (one config per
dispatch, because a single dispatch carrying both died at 69 min on rpc DeadlineExceeded and lost the
second config plus the single end-of-run tar). This extracts both census JSONs and diffs them.

THE VALIDITY GATE IS THE FIRST THING PRINTED. If A and B do not both show a complete, parseable census
with the expected refusals, the comparison is VOID and says so, rather than reporting a delta that is
really a truncated log or a config that never built. An earlier lane conclusion had to be withdrawn for
exactly this class of error.

WHAT THE SPLIT MEANS:
  ADMITTED  = refuses in A, absent from B's refusals. Must be a SUBSET of the rows probe v4 says the
              predicate answers true for (ownBound=1, 141 of 254). Rows outside that set mean the arm
              fired somewhere the probe did not predict: that is reported, not smoothed over.
  NEWLY     = absent from A's refusals, refuses in B. Should be EMPTY -- the arms only ever accept, never
              refuse. A nonzero count here is a bug in my placement, not a finding to celebrate.
Both are listed by identity (formal, actual, span), never just counted.
"""
import base64
import io
import json
import os
import re
import subprocess
import sys
import tarfile

LOG_A = sys.argv[1] if len(sys.argv) > 1 else '/tmp/abA2_dispatch.log'
LOG_B = sys.argv[2] if len(sys.argv) > 2 else '/tmp/abB_dispatch.log'
WORK = '/tmp/ab_split'


def census_from(logpath, name):
    """Return (rows, meta) or (None, why) -- never a silently empty list."""
    if not os.path.exists(logpath):
        return None, "log missing: %s" % logpath
    txt = open(logpath, errors='replace').read()
    m = re.search(r'=== BEGIN-TAR ===\s*(.*?)\s*=== END-TAR ===', txt, re.S)
    if not m:
        # Distinguish 'the run died before shipping' from 'the run shipped nothing' -- both look like an
        # absent tar, and conflating them would let a failed build pass as an empty census.
        died = re.findall(r'NO CANDIDATE[^\n]*|rpc DeadlineExceeded|CLONE FAILED|FETCH FAILED|abort \(', txt)
        return None, "no tar in log%s" % (" (run aborted: %s)" % died[-1] if died else "; tail: " + txt.strip().splitlines()[-1][:160])
    try:
        raw = base64.b64decode(''.join(m.group(1).split()))
        tf = tarfile.open(fileobj=io.BytesIO(raw))
    except Exception as e:
        return None, "tar unreadable: %s" % e
    names = tf.getnames()
    want = 'census_%s.json' % name
    if want not in names:
        return None, "tar lacks %s (has %s)" % (want, names[:8])
    out = os.path.join(WORK, name)
    os.makedirs(out, exist_ok=True)
    tf.extractall(out)
    p = os.path.join(out, want)
    try:
        rows = json.load(open(p))
    except Exception as e:
        return None, "census JSON unreadable: %s" % e
    if not isinstance(rows, list) or not rows:
        return None, "census is empty (%r) -- a self-host that refused to RUN also produces this, so it is not treated as 'no refusals'" % (type(rows),)
    meta = {'selfhost_rc': (re.search(r'self-host rc=(\d+)', txt) or [None, '?'])[1],
            'rebind_lines': len(re.findall(r'\bbound as\b', txt)),
            'rows': len(rows)}
    return rows, meta


def ident(r):
    return (r.get('formal', ''), r.get('actual', ''), r.get('span', ''))


A, mA = census_from(LOG_A, 'A')
B, mB = census_from(LOG_B, 'B')
print("A: %r" % (mA,))
print("B: %r" % (mB,))
if A is None or B is None:
    print("\nCOMPARISON VOID -- no delta is reported. A=%s B=%s"
          % ('OK' if A else mA, 'OK' if B else mB))
    sys.exit(3)

ia = {ident(r) for r in A}
ib = {ident(r) for r in B}
admitted = sorted(ia - ib)
newly = sorted(ib - ia)
print("\nrows: A=%d distinct=%d   B=%d distinct=%d" % (len(A), len(ia), len(B), len(ib)))
print("ADMITTED by the arms (in A's refusals, gone in B) = %d" % len(admitted))
print("NEWLY REFUSED (should be 0; the arms only accept) = %d" % len(newly))
for i, k in enumerate(admitted):
    print("  - %-28s %-46s %s" % ((k[0] or '')[:28], (k[1] or '')[:46], (k[2] or '')[:52]))
    if i >= 59:
        print("   ... (%d more)" % (len(admitted) - 60)); break
for k in newly[:20]:
    print("  + %-28s %-46s %s" % ((k[0] or '')[:28], (k[1] or '')[:46], (k[2] or '')[:52]))

# THE SUBSET CHECK against probe v4's ownBound rows, if the v4 log is available.
v4 = '/tmp/probeV4d_dispatch.log'
if os.path.exists(v4) and admitted:
    txt = open(v4, errors='replace').read()
    fires = set()
    for line in txt.splitlines():
        if not line.startswith('REBINDV4|'):
            continue
        kv = dict(p.split('=', 1) for p in line[len('REBINDV4|'):].split('|') if '=' in p)
        if kv.get('ownBound') == '1':
            fires.add((kv.get('formal', ''), kv.get('lshape', ''), kv.get('module', '')))
    # census identity is (formal, actual, span); v4 carries module+shapes. Match on formal+actual only,
    # because the census span is a source span and v4 has none -- and SAY SO rather than implying a join
    # stronger than it is.
    outside = [k for k in admitted
               if not any(k[0] == f and (k[1] or '').startswith((s or '')[:40]) for f, s, _ in fires)]
    print("\nSUBSET CHECK vs probe v4 ownBound=1 (%d rows): admitted rows NOT in that set = %d"
          % (len(fires), len(outside)))
    print("  (joined on formal + actual-prefix only: the census has source spans, v4 does not -- a"
          " stricter join needs v4's per-row module, which this run only aggregated)")
    for k in outside[:12]:
        print("   OUTSIDE: %-28s %-46s" % ((k[0] or '')[:28], (k[1] or '')[:46]))
json.dump({'admitted': admitted, 'newly_refused': newly,
           'A_distinct': len(ia), 'B_distinct': len(ib)},
          open('/tmp/ab_split_result.json', 'w'), indent=1)
print("\nwrote /tmp/ab_split_result.json")
