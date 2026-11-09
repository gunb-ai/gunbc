#!/bin/bash
# scratch (NOT for commit): does the fci1 witness closure fail the SAME way on unmodified origin/main?
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgt
[ -x /tmp/tgt/release/claim_batch ] || RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccache ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch || exit 1
rm -rf /tmp/mainsrc && git clone -q --no-hardlinks /root/workspace /tmp/mainsrc 2>/dev/null || { echo "CLONE FAILED"; exit 1; }
git -C /tmp/mainsrc checkout -q origin/main || { echo "CHECKOUT FAILED"; exit 1; }
git -C /tmp/mainsrc log --oneline -1 | cat
f=dag/test/claim/fabric/fabric_ci_fci_fci1_placeholder.dag
f=dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "/tmp/mainsrc/$f" | paste -sd,)
echo "MAIN has $(echo $fns | tr ',' '\n' | wc -l) test fns in the fci1 lifecycle witness"
/tmp/tgt/release/claim_batch --source-root /tmp/mainsrc/dag --source-root /tmp/mainsrc/src/v2 --entry "$f" --functions "$fns" > /tmp/main_fci1.log 2>&1
echo "MAIN RESULT exit=$? PASS=$(grep -c '^PASS' /tmp/main_fci1.log) FAIL=$(grep -c '^FAIL' /tmp/main_fci1.log)"
grep '^FAIL' /tmp/main_fci1.log | cut -c1-300 | head -6
echo "ATTRIB DONE"
