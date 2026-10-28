#!/usr/bin/env bash
# TEST A: head + bootstrap, WITHOUT the working-tree edits (the dispatch ships the tree as-is;
# this script reverts the two source files to head first). If 1430 refusal still happens it is
# branch-caused, not edit-caused.
set -uo pipefail
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
cd "$(dirname "$0")"
git checkout HEAD -- src/v1/05_emit_rust.dag dag/gunbc/recurring_failure_mode/one_name_two_representations_kernel_map_and_finitely_supported_function.dag
git checkout ae563700cbe -- src/v1/stage0/src/std_algebra.rs src/v1/stage0/src/std_types.rs src/v1/stage0/src/std_computation.rs
echo "BOOTSTRAPPED:"; git status --porcelain | head -8
echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|Finished" | head -8
echo "=== REGEN ==="
./target/release/claim_executor --required-regen \
  --regen-candidate-dir target/stage0-regen-candidate \
  --regen-receipt target/stage0-regen-receipt.json \
  --source-root dag --source-root src/v2 > /tmp/regen-testA.out 2>&1
echo "REGEN_EXIT=$?"
grep -c "effect summary incomplete" /tmp/regen-testA.out | sed 's/^/EFFECT_SUMMARY_INCOMPLETE=/'
grep -E "^required-regen|refused" /tmp/regen-testA.out | head -6
echo "=== DIAGNOSTIC SURVEY ==="
sed 's/(src.*//; s/(dag.*//' /tmp/regen-testA.out | grep -E "^[a-z].*: " | sed 's/: .*//' | sort | uniq -c | sort -rn | head -12
echo "--- distinct callee modules:"
grep -o "calls [a-z0-9_.]*\." /tmp/regen-testA.out | sort | uniq -c | sort -rn | head -12
echo "--- non-effect-summary hard diagnostics:"
grep -vE "effect summary incomplete|^✓|^◐|memory-cgroup" /tmp/regen-testA.out | grep -E "^[a-z].*: " | head -12
