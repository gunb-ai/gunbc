#!/bin/bash
# scratch (NOT for commit): post-rebase re-verification of the allocation witness alone.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgt
[ -x /tmp/tgt/release/claim_batch ] || RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccache ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch || exit 1
f=dag/test/claim/runner/runner_slot_allocation_witness.dag
fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
/tmp/tgt/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > /tmp/alloc_postrebase.log 2>&1
echo "RESULT allocation exit=$? PASS=$(grep -c '^PASS' /tmp/alloc_postrebase.log) FAIL=$(grep -c '^FAIL' /tmp/alloc_postrebase.log)"
grep '^FAIL' /tmp/alloc_postrebase.log | cut -c1-300
echo "ALLOC DONE"
