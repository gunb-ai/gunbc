#!/usr/bin/env bash
# TEST B: ALL mirrors from main (a coherent compiler) + HEAD's live corpus (merged .dag files,
# no working-tree edits). TEST A (head compiler mirrors + main std-core) refused with 1430
# effect-summary diagnostics; the control (all main, main corpus) was clean. If TEST B is clean,
# the refusal lives in the stale head-side compiler mirrors and a regen from main's compiler
# over the merged corpus fixes exactly that. If TEST B refuses with 1430, the merged CORPUS
# itself breaks the effect join under main's compiler -- a corpus-side repair, different work.
set -uo pipefail
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
cd "$(dirname "$0")"
git checkout HEAD -- src/v1/05_emit_rust.dag dag/gunbc/recurring_failure_mode/one_name_two_representations_kernel_map_and_finitely_supported_function.dag
git checkout ae563700cbe -- src/v1/stage0/src/
echo "BOOTSTRAPPED (all mirrors = main):"; git status --porcelain | grep stage0/src | wc -l | sed 's/^/modified_mirrors=/'
echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|Finished" | head -8
echo "=== REGEN over merged corpus ==="
./target/release/claim_executor --required-regen \
  --regen-candidate-dir target/candB \
  --regen-receipt target/receiptB.json \
  --source-root dag --source-root src/v2 > /tmp/regen-testB.out 2>&1
echo "REGEN_EXIT=$?"
grep -c "effect summary incomplete" /tmp/regen-testB.out | sed 's/^/EFFECT_SUMMARY_INCOMPLETE=/'
grep -E "^required-regen" /tmp/regen-testB.out | head -4
