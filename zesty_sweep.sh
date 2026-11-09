#!/bin/bash
# scratch (NOT for commit): run every witness file this PR touches or consumes.
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
  n=$(echo "$fns" | tr ',' '\n' | wc -l)
  start=$(date +%s)
  /tmp/tgt/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" \
    --functions "$fns" > "/tmp/zestylogs/$b.log" 2>&1
  code=$?
  p=$(grep -c '^PASS' "/tmp/zestylogs/$b.log"); q=$(grep -c '^FAIL' "/tmp/zestylogs/$b.log")
  echo "RESULT $b fns=$n exit=$code PASS=$p FAIL=$q elapsed=$(( $(date +%s) - start ))s"
  grep '^FAIL' "/tmp/zestylogs/$b.log" | sed 's/^/    /'
  grep -oP '(?<=closure diagnostics in ).*' "/tmp/zestylogs/$b.log" | sort -u | cut -c1-260 | sed 's/^/    DIAG /'
}
for f in \
  dag/test/claim/fabric/fabric_cell_converge_witness.dag \
  dag/test/claim/runner/runner_capacity_realize_witness.dag \
  dag/test/claim/runner/runner_slot_retirement_witness.dag \
  dag/test/claim/fabric/fabric_cell_observe_witness.dag
do
  [ -f "$f" ] && run_one "$f" || echo "MISSING $f"
done
echo "SWEEP DONE"
