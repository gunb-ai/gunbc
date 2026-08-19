set -e
cd .

echo "=== BEFORE PHASE (sort_by row still broken) ==="
git stash push -- dag/std/algebra.dag
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -8

set +e
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet > /tmp/before_mw.log 2>&1
echo "before_mw_exit=$?"
set -e
echo "before error count:"; grep -c "error:" /tmp/before_mw.log || true
grep "error:" /tmp/before_mw.log | sort > /tmp/before_mw_errors.sorted

echo "=== AFTER PHASE (sort_by row fixed via stash pop) ==="
git stash pop

echo "--- regenerate candidate std_algebra.rs from fixed algebra.dag using the BEFORE binary ---"
set +e
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 --regen-candidate-dir /tmp/algebra-regen-candidate --regen-receipt /tmp/algebra-regen-receipt.json > /tmp/algebra_rr.log 2>&1
echo "algebra_rr_exit=$?"
set -e
tail -15 /tmp/algebra_rr.log

CAND=$(find /tmp/algebra-regen-candidate -iname "std_algebra.rs" | head -1)
echo "candidate file: $CAND"
if [ -n "$CAND" ]; then
  diff "$CAND" src/v1/stage0/src/std_algebra.rs || true
  cp "$CAND" src/v1/stage0/src/std_algebra.rs
  echo "applied std_algebra.rs"
else
  echo "NO CANDIDATE FOUND — cannot apply regen"
  exit 97
fi

cargo fmt --all 2>&1 | tail -5 || true

echo "--- rebuild with fixed std_algebra.rs ---"
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -10

echo "--- after-fix main_wet ---"
set +e
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet > /tmp/after_mw.log 2>&1
echo "after_mw_exit=$?"
set -e
echo "after error count:"; grep -c "error:" /tmp/after_mw.log || true
grep "error:" /tmp/after_mw.log | sort > /tmp/after_mw_errors.sorted

echo "=== DISCRIMINATING DIFF: before vs after ==="
diff /tmp/before_mw_errors.sorted /tmp/after_mw_errors.sorted || true

echo "=== int-vs-string family check (must remain fixed / absent) ==="
grep -i "linux.dag\|proc_self_cgroup\|cache_interface\|domain_name" /tmp/after_mw.log || echo "none — good, still clean"

echo "=== remaining method-arg-contract-unavailable refusals ==="
grep -c "no declared argument contract is available" /tmp/after_mw.log || true
grep "no declared argument contract is available" /tmp/after_mw.log

echo "DONE"
