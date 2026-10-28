#!/usr/bin/env bash
# CONTROL: the handoff's exact regen command, at main (a green head). If main refuses with the
# same 1430 effect-summary diagnostics, the refusal is pre-existing and not caused by the branch.
set -uo pipefail
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
cd "$(dirname "$0")"
echo "HEAD=$(git rev-parse HEAD)"
echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|Finished" | head -8
echo "=== REGEN ==="
./target/release/claim_executor --required-regen \
  --regen-candidate-dir target/stage0-regen-candidate \
  --regen-receipt target/stage0-regen-receipt.json \
  --source-root dag --source-root src/v2 > /tmp/regen-control.out 2>&1
echo "REGEN_EXIT=$?"
grep -c "effect summary incomplete" /tmp/regen-control.out | sed 's/^/EFFECT_SUMMARY_INCOMPLETE=/'
head -25 /tmp/regen-control.out
echo "=== DIFF vs COMMITTED ==="
diff -rq target/stage0-regen-candidate/src src/v1/stage0/src 2>/dev/null | grep -v "^Only in src" | head -10
