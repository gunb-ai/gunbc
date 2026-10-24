#!/bin/bash
set -u
source <(sed -n '/^B()/p;/^regen()/,/^}/p' tools/.phaseb_run.sh)
B; regen
for d in tools/.phb_src/*/; do
  echo "=== $d"
  timeout 600 ./target/release/gunbc run --source-root $d --source-root dag --entry $d/p.dag --function text 2>&1 | head -12 | cut -c1-300
done
