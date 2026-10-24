#!/bin/bash
set -u
source <(sed -n '/^B()/p;/^regen()/,/^}/p' tools/.phaseb_run.sh)
B; regen
git diff --stat -- src/v1/stage0
echo "MIRRORPATCH-BEGIN"; git diff --binary -- src/v1/stage0 | gzip -9 | base64 -w0; echo; echo "MIRRORPATCH-END"
