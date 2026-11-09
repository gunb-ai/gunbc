#!/bin/bash
# scratch (NOT for commit): full closure diagnostics for the fci1 witness.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/session-home/target-zesty RUSTC_WRAPPER= SCCACHE_DIR=/session-home/sccache
ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch || { echo "BUILD FAILED"; exit 1; }
f=dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
/session-home/target-zesty/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > /session-home/logs/fci1_full.log 2>&1
echo "RC=$? PASS=$(grep -c '^PASS' /session-home/logs/fci1_full.log) FAIL=$(grep -c '^FAIL' /session-home/logs/fci1_full.log)"
grep '^FAIL' /session-home/logs/fci1_full.log | head -2
echo "FCI1L DONE"
