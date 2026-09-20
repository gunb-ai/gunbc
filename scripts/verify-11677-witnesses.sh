#!/usr/bin/env bash
# One-off verification driver for gunbc#11677 at ec2973a0f24.
#
# WHY THIS EXISTS AND WHY IT IS NOT IN #11677: the required floor lane exits 0
# over a parse failure (run 35494972594, job 106036217104, step 8 => success
# while logging "FAILED PHASE parse (8 error(s))"), so CI's green is not a
# witness verdict for this head. This runs the three witness files that judge
# the change, on a host with memory, and prints their output verbatim.
#
# It lives on branch verify/keen-bear-791-ec2973a0, which is ec2973a0f24 plus
# this file, so the driver sits INSIDE the repo (gunbc refuses a source root
# outside it) and the tree under test is the PR head unchanged.
set -uo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "=== head: $(git rev-parse HEAD)"
echo "=== tree differs from ec2973a0f24 only by this driver:"
git diff --stat ec2973a0f24 HEAD || true

MEM_MAX="${MEM_MAX:-24G}"
CB="$ROOT/target/release/claim_batch"

echo "=== building claim_batch (release)"
cargo build --release -p v1-compiler --bin claim_batch || { echo "BUILD FAILED"; exit 9; }

OUT="$(mktemp -d)"
run_file() {
  entry="$1"
  tag="$(basename "$entry" .dag)"
  fns="$(grep -oP '^test fn \K\w+' "$entry" | paste -sd,)"
  echo
  echo "############ $entry"
  echo "functions: $fns"
  systemd-run --user --scope -p MemoryMax="$MEM_MAX" --quiet \
    "$CB" --source-root dag --source-root src/v2 --entry "$entry" --functions "$fns" --hermetic > "$OUT/$tag.log" 2>&1
  rc=$?
  cat "$OUT/$tag.log"
  echo "############ exit=$rc for $entry"
}

run_file dag/test/claim/runner/runner_attempt_launch_witness_test.dag
run_file dag/test/claim/github_effect_perform_witness_test.dag
run_file dag/test/claim/runner/runner_microvm_lifecycle_witness_test.dag

echo
echo "=== SUMMARY (PASS/FAIL/refusal lines only)"
grep -hE "^(PASS|FAIL)|error|refused|not found" "$OUT"/*.log | cut -c1-240 || true
echo
echo "=== COUNTS"
echo "PASS: $(grep -hcE '^PASS' "$OUT"/*.log | paste -sd+ | bc 2>/dev/null || echo 0)"
echo "FAIL: $(grep -hcE '^FAIL' "$OUT"/*.log | paste -sd+ | bc 2>/dev/null || echo 0)"
echo "(a MemoryStallRefusedPageThrash line is the tool refusing to trust its own"
echo " measurement under host pressure -- that is no verdict, not a pass.)"
