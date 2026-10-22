#!/bin/bash
# Scratch (never lands): prA lane E2b = the 3 NEW-file claims. E2 (be056e34) lost them:
# its own `git clean -x -d --force` deleted ctrl-build's applied patch of the UNTRACKED
# new files (claim_batch then refused: file does not exist), while tracked-file edits
# (mirrors, re-spelled claims) survived and those four claims passed rc=0.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== HEAD ==="
git rev-parse --short HEAD
for f in src/v2/test/claim/manual/unary_prefix_values.dag src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag; do
  ls -la $f || { echo "FATAL: new file missing on runner"; exit 4; }
done
echo "mirror witnesses: find_prefix_binding_power=$(grep -c find_prefix_binding_power src/v1/stage0/src/v1_compiler_parse.rs) literal-min-bp=$(grep -c 'min_bp: 1[0-9]' src/v1/stage0/src/v1_compiler_parse.rs)"

echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1

run_claim() { # $1 label $2 entry $3 fn
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "$2" --function "$3" > /tmp/claim_$1.log 2>&1
  local rc=$?
  echo "claim@$1 rc=$rc"
  grep -aE "✗|refus|DISAGREE|error|panic" /tmp/claim_$1.log | head -4
  tail -1 /tmp/claim_$1.log | sed 's/^/  last: /'
  return 0
}

run_claim values_arms src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim values_emitted src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_emitted_text_follows_the_tree
run_claim agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
echo "=== DONE ==="
