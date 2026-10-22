#!/bin/bash
set -uo pipefail
PIN=e47afdde7fbf624eecf68b3d115498702f6d437f
CUR=$(git rev-parse HEAD)
if [ "$CUR" != "$PIN" ]; then echo "PIN: was $CUR"; git checkout -f -q "$PIN" || { echo PIN-FAILED; exit 3; }; fi
echo STEP build; date -u +%H:%M:%S
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
for pair in "recshape_v1_tmp recshape_v1_reports RECSHAPE-V1" "recshape_v2_tmp recshape_v2_reports RECSHAPE-V2"; do
  set -- $pair
  echo "STEP $1"; date -u +%H:%M:%S
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "src/v2/test/claim/manual/$1.dag" --function "$2" > /tmp/rs_$1.log 2>&1
  echo "rc=$?"
  sed -n "/FAIL $2/,/\[witness\]/p" /tmp/rs_$1.log | cut -c1-300
done
echo "=== DONE ==="
