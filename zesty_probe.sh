#!/bin/bash
# scratch (NOT for commit): verification of the probe-side effect-binding change (commit 2).
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgt
if [ ! -x /tmp/tgt/release/claim_batch ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccache ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch || {
    echo "BUILD FAILED"; exit 1; }
fi
mkdir -p /tmp/zestylogs
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "$b SKIP-no-fns"; return; }
  /tmp/tgt/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" \
    --functions "$fns" > "/tmp/zestylogs/$b.log" 2>&1
  echo "RESULT $b exit=$? PASS=$(grep -c '^PASS' /tmp/zestylogs/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zestylogs/$b.log)"
  grep '^FAIL' "/tmp/zestylogs/$b.log" | sed 's/^/  RED /'
}
run_one dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_bounded_controls_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_claim_contract_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_claim_run_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_retry_refusal_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_release_purity_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_release_rewind_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_observation_witness.dag
echo "PROBE DONE"
