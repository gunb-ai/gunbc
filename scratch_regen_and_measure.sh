set -e
echo "AMBIENT RUSTC_WRAPPER before clearing: '${RUSTC_WRAPPER:-<unset>}'"
unset RUSTC_WRAPPER || true
export RUSTC_WRAPPER=
MERGE_FULL=1fba7e6c5e69e9c2bfe92573553c648242fe17c1
echo "MERGE_FULL=$MERGE_FULL"

git fetch origin $MERGE_FULL
git checkout --force $MERGE_FULL
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$MERGE_FULL" ]; then echo "BASEFAIL got $GOT want $MERGE_FULL"; exit 99; fi
echo "MERGE_OK $GOT"

echo "=== build pre-regen binary from merge head (stale .rs mirrors, but this is only the regen TOOL, not the measured artifact) ==="
rm -rf /tmp/target-preregen
cargo build --release --target-dir /tmp/target-preregen -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -15

echo "=== required-regen: produce corrected std_algebra.rs and v1_compiler_infer.rs ==="
set +e
/tmp/target-preregen/release/claim_executor --required-regen --source-root dag --source-root src/v2 --regen-candidate-dir /tmp/merge-regen-candidate --regen-receipt /tmp/merge-regen-receipt.json > /tmp/merge_rr.log 2>&1
RR_EXIT=$?
echo "merge_rr_exit=$RR_EXIT"
set -e
tail -40 /tmp/merge_rr.log

ALGEBRA_CAND=$(find /tmp/merge-regen-candidate -iname "std_algebra.rs" | head -1)
INFER_CAND=$(find /tmp/merge-regen-candidate -iname "v1_compiler_infer.rs" | head -1)
echo "algebra candidate: $ALGEBRA_CAND"
echo "infer candidate: $INFER_CAND"

if [ -n "$ALGEBRA_CAND" ]; then
  diff "$ALGEBRA_CAND" src/v1/stage0/src/std_algebra.rs || true
  cp "$ALGEBRA_CAND" src/v1/stage0/src/std_algebra.rs
  echo "applied std_algebra.rs"
else
  echo "NO CANDIDATE for std_algebra.rs -- ABORTING, this must not be hand-mirrored silently"
  exit 97
fi

if [ -n "$INFER_CAND" ]; then
  diff "$INFER_CAND" src/v1/stage0/src/v1_compiler_infer.rs || true
  cp "$INFER_CAND" src/v1/stage0/src/v1_compiler_infer.rs
  echo "applied v1_compiler_infer.rs"
else
  echo "NO CANDIDATE for v1_compiler_infer.rs -- ABORTING, this must not be hand-mirrored silently"
  exit 96
fi

cargo fmt --all 2>&1 | tail -5 || true

echo "=== REGEN DIFF (capture for local re-application; this checkout has no push creds assumed) ==="
git diff -- src/v1/stage0/src/std_algebra.rs src/v1/stage0/src/v1_compiler_infer.rs > /tmp/regen_mirrors.diff
wc -l /tmp/regen_mirrors.diff
echo "--- BEGIN REGEN_MIRRORS_DIFF ---"
cat /tmp/regen_mirrors.diff
echo "--- END REGEN_MIRRORS_DIFF ---"

echo "=== rebuild with corrected mirrors (the real 'after' artifact) ==="
rm -rf /tmp/target-final
cargo build --release --target-dir /tmp/target-final -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -20

echo "=== run main_wet on the FINAL (post-merge, post-regen) binary -- this is the AFTER measurement ==="
set +e
/tmp/target-final/release/gunbc run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet > /tmp/final_after_mw.log 2>&1
echo "final_after_mw_exit=$?"
set -e
echo "final after error count:"; grep -c "error:" /tmp/final_after_mw.log || true
grep "error:" /tmp/final_after_mw.log | sort > /tmp/final_after_mw_errors.sorted

echo "=== now build the BEFORE baseline: origin/main tip (pre-fix, pre-merge) ==="
MAIN_TIP_FULL=c4642e0a8ae42a25dbfb7e34e73e9db3a29ea51e
git fetch origin main
MAIN_ACTUAL=$(git rev-parse origin/main)
echo "origin/main actual tip: $MAIN_ACTUAL (expected roughly $MAIN_TIP_FULL)"
git checkout --force $MAIN_ACTUAL
rm -rf /tmp/target-mainbase
cargo build --release --target-dir /tmp/target-mainbase -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -15
set +e
/tmp/target-mainbase/release/gunbc run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet > /tmp/before_main_mw.log 2>&1
echo "before_main_mw_exit=$?"
set -e
echo "before(main tip) error count:"; grep -c "error:" /tmp/before_main_mw.log || true
grep "error:" /tmp/before_main_mw.log | sort > /tmp/before_main_mw_errors.sorted

echo "=== CLEARED (in main-tip before, not in final after) ==="
comm -23 /tmp/before_main_mw_errors.sorted /tmp/final_after_mw_errors.sorted | tee /tmp/cleared_final.txt | wc -l
echo "=== SURVIVING (in both) ==="
comm -12 /tmp/before_main_mw_errors.sorted /tmp/final_after_mw_errors.sorted | tee /tmp/surviving_final.txt | wc -l
echo "=== NEWLY APPEARING (in final after, not in main-tip before) ==="
comm -13 /tmp/before_main_mw_errors.sorted /tmp/final_after_mw_errors.sorted | tee /tmp/newly_final.txt | wc -l

echo "=== sample CLEARED ==="; head -40 /tmp/cleared_final.txt
echo "=== sample SURVIVING ==="; head -40 /tmp/surviving_final.txt
echo "=== full NEWLY APPEARING ==="; cat /tmp/newly_final.txt

echo "=== newly-appearing split: contract-unavailable-coverage-gap vs other ==="
grep -c "method-arg-contract-unavailable\|no declared argument contract is available" /tmp/newly_final.txt || true
grep "method-arg-contract-unavailable\|no declared argument contract is available" /tmp/newly_final.txt || echo "none"

echo "DONE"
