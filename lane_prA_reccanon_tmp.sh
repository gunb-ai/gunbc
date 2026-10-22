#!/bin/bash
set -uo pipefail
PIN=e47afdde7fbf624eecf68b3d115498702f6d437f
CUR=$(git rev-parse HEAD)
if [ "$CUR" != "$PIN" ]; then echo "PIN: was $CUR"; git checkout -f -q "$PIN" || { echo PIN-FAILED; exit 3; }; fi
echo STEP build; date -u +%H:%M:%S
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
run() {
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "src/v2/test/claim/manual/$1.dag" --function "$2" > /tmp/$3.log 2>&1
  echo "rc=$?"; sed -n "/FAIL $2/,/\[witness\]/p" /tmp/$3.log | grep -vE 'resolve-split|assembly-split|cost-partition|process-partition|pre-entry|GiB' | cut -c1-300
}
run record_canon_v1_tmp record_canon_v1_reports rc_v1
run record_canon_v2_tmp record_canon_v2_reports rc_v2
echo "=== DONE ==="
