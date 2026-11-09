#!/bin/bash
# scratch (NOT for commit): failure detail for the two rows still red on this branch.
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
  for t in $2; do
    echo "---- DETAIL $t ----"
    awk -v t="$t" 'index($0, "FAIL " t) == 1 {f=1} f {print; c++} c>18 {exit}' "/tmp/zestylogs/$b.log"
  done
}
run_one dag/test/claim/fabric/fabric_cell_converge_witness.dag witness_fabric_cell_neighbours_outside_the_roster_are_refused
run_one dag/test/claim/runner/runner_capacity_realize_witness.dag apply_seam_plans_additions_at_the_committed_width_not_count_refusal
echo "DETAIL DONE"
