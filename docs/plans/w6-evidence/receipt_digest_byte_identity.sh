#!/bin/sh
# W6 verification: the in-substrate receipt reconstruction is BYTE-IDENTICAL to the bash
# approval_broker_tree_digest_expr. Run from the repo root. Both must print the same hex.
#
#   bash:  cd <root> && find dag src/v2 -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum | cut -d' ' -f1
#   typed: per file: hex=sha256(file); line = "<hex>  <relpath>\n"; sort lines BY RELPATH (bytewise);
#          digest = sha256(concat of lines)
#
# IMPORTANT semantics: `sort -z` sorts the FILE LIST (NUL-delimited relpaths), not the hash lines.
# So the sort key is the RELPATH, not "<hex>  <relpath>". This matters when one relpath is a prefix
# of another and the two hex prefixes order them oppositely (see the discriminating fixture in
# W6_HANDOFF.md; a fixture of that shape gives different digests for the two keys).
# The C locale is what the deployment host runs (LANG=C.UTF-8 per the ntfy readback witness) and
# what every other tree-manifest digest in this repo binds (LC_ALL=C):
#   runner_browser_toolchain.dag:477, self_host_compile_phase_frontier.dag:581, gnu_coreutils.dag:548
set -e
export LC_ALL=C
echo "bash   = $(find dag src/v2 -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum | cut -d' ' -f1)"
python3 - <<'PY'
import os,hashlib
relhex=[]
for sub in ("dag","src/v2"):
    for dp,dn,fn in os.walk(sub):
        for f in fn:
            rel=os.path.join(dp,f)
            relhex.append((rel,hashlib.sha256(open(rel,'rb').read()).hexdigest()))
body="".join(h+"  "+r+"\n" for r,h in sorted(relhex,key=lambda t:t[0].encode()))
print("typed  = "+hashlib.sha256(body.encode()).hexdigest())
print("files  = %d" % len(relhex))
PY
