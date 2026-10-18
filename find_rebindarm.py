#!/usr/bin/env python3
"""Recover probe v2's REBINDARM rows from where they actually are.

They are NOT in the wrapper dispatch log: the script prints them inside the guest and ships them
base64-encoded in the === BEGIN-TAR === block. So `grep REBINDARM <dispatch>.log` returns 0 and invites
the wrong conclusion that the probe never reached the refusal site. That is an instrument error, not a
measurement. Decode every tar in every candidate log and count real rows.
"""
import sys, re, base64, io, tarfile, glob

out = open('/tmp/rebindarm_recovered.txt', 'w')
pat = re.compile(r'=== BEGIN-TAR ===\s*(.*?)\s*=== END-TAR ===', re.S)
total = 0
for p in sorted(glob.glob('/tmp/probe*.log') + glob.glob('/tmp/armb*.log') + glob.glob('/tmp/mainprobe*.log')):
    try:
        txt = open(p, errors='replace').read()
    except Exception:
        continue
    n = 0
    for b in pat.findall(txt):
        try:
            raw = base64.b64decode(''.join(b.split()))
            tf = tarfile.open(fileobj=io.BytesIO(raw))
        except Exception:
            continue
        for m in tf.getmembers():
            if not m.isfile():
                continue
            try:
                d = tf.extractfile(m).read().decode('utf-8', 'replace')
            except Exception:
                continue
            c = d.count('REBINDARM|bind=')
            if c:
                n += c
                out.write(d)
    if n:
        print("  %-44s rows=%d" % (p, n))
        total += n
out.close()
print("TOTAL rows recovered: %d -> /tmp/rebindarm_recovered.txt" % total)
