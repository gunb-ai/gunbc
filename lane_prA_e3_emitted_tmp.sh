#!/bin/bash
# Scratch (never lands): prA lane E3 = re-run of the corrected emitted-text claim (the
# oracle's read path must name the EMITTED file, module dots->underscores; executed
# refusal 6f65817b named the emitted set: src/probe_unary_prefix_minus.rs et al).
# NO git clean (E2b proved it deletes ctrl-build's patched untracked files).
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }

echo "=== HEAD ==="
git rev-parse --short HEAD
grep -c "probe_unary_prefix_" src/v2/test/claim/manual/unary_prefix_values.dag | sed 's/^/corrected-emit-paths: /'

echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1

echo "--- values_emitted (corrected) ---"
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_values.dag --function unary_prefix_emitted_text_follows_the_tree > /tmp/claim_values_emitted3.log 2>&1
echo "claim@values_emitted3 rc=$?"
grep -aE "refus|✗|error|panic|names no module" /tmp/claim_values_emitted3.log | head -4
tail -1 /tmp/claim_values_emitted3.log | sed 's/^/  last: /'

echo "--- values_arms + agreement re-confirm on this tree ---"
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_values.dag --function unary_prefix_value_arms_hold > /tmp/claim_values_arms3.log 2>&1
echo "claim@values_arms3 rc=$?"
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag --function unary_prefix_v1_v2_agreement_reports > /tmp/claim_agreement3.log 2>&1
echo "claim@agreement3 rc=$?"
echo "=== DONE ==="
