set -e
BASE_SHA=2022973f9769de14ac891e591fd466d127f261dc
FIX_SHA=414131345ea2b7d99e66b36eb4658d19d5424166

echo "=== BASELINE PHASE ==="
git fetch origin $BASE_SHA
git checkout --force $BASE_SHA
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$BASE_SHA" ]; then echo "BASEFAIL got $GOT want $BASE_SHA"; exit 99; fi
echo "BASE_OK $GOT"
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -5

echo "--- baseline main_wet ---"
set +e
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet > /tmp/base_mw.log 2>&1
echo "base_mw_exit=$?"
set -e
grep -c "error:" /tmp/base_mw.log || true
grep "error:" /tmp/base_mw.log | sort > /tmp/base_mw_errors.sorted

echo "--- baseline required-regen (candidate build only, expected to FAIL on pre-existing unrelated surface mismatch) ---"
set +e
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 --regen-candidate-dir /tmp/base-regen-candidate --regen-receipt /tmp/base-regen-receipt.json > /tmp/base_rr.log 2>&1
echo "base_rr_exit=$?"
set -e
tail -15 /tmp/base_rr.log

echo "=== FIX PHASE ==="
git fetch origin $FIX_SHA
git checkout --force $FIX_SHA
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$FIX_SHA" ]; then echo "FIXFAIL got $GOT want $FIX_SHA"; exit 98; fi
echo "FIX_OK $GOT"

echo "--- regenerate candidate stage0 rust from fixed .dag using the BASELINE binary (already built above) ---"
set +e
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 --regen-candidate-dir /tmp/fix-regen-candidate --regen-receipt /tmp/fix-regen-receipt.json > /tmp/fix_rr_gen.log 2>&1
echo "fix_rr_gen_exit=$?"
set -e
tail -15 /tmp/fix_rr_gen.log

echo "--- diff candidate infer/lookup rust vs committed (confirm real, non-trivial emitted change) ---"
find /tmp/fix-regen-candidate -iname "*infer*.rs" -o -iname "*lookup*.rs" | while read f; do
  rel=${f#/tmp/fix-regen-candidate/}
  committed=$(find src/v1/stage0/src -name "$(basename "$f")")
  if [ -n "$committed" ]; then
    echo "=== diff stat: $f vs $committed ==="
    diff -q "$f" "$committed" || true
  fi
done

echo "--- apply ONLY the infer/lookup candidate files into stage0 (scoped apply; leave the 8 unrelated surface-mismatch files untouched) ---"
for f in $(find /tmp/fix-regen-candidate -iname "*infer*.rs" -o -iname "*lookup*.rs"); do
  bn=$(basename "$f")
  target=$(find src/v1/stage0/src -name "$bn")
  if [ -n "$target" ]; then
    cp "$f" "$target"
    echo "applied $bn -> $target"
  else
    echo "NOTE: no committed counterpart found for $bn (candidate-only) — not applying"
  fi
done

cargo fmt --all 2>&1 | tail -5 || true

echo "--- rebuild gunbc+claim_executor with fixed stage0 rust ---"
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -10

echo "--- fixed-binary main_wet ---"
set +e
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet > /tmp/fix_mw.log 2>&1
echo "fix_mw_exit=$?"
set -e
grep -c "error:" /tmp/fix_mw.log || true
grep "error:" /tmp/fix_mw.log | sort > /tmp/fix_mw_errors.sorted

echo "=== DISCRIMINATING DIFF: baseline vs fixed main_wet errors ==="
diff /tmp/base_mw_errors.sorted /tmp/fix_mw_errors.sorted || true

echo "=== method-arg-contract-unavailable refusal count in fixed run ==="
grep -c "method-arg-contract-unavailable" /tmp/fix_mw.log || true
grep "method-arg-contract-unavailable" /tmp/fix_mw.log | head -20 || true

echo "=== linux.dag:159 area, baseline vs fixed ==="
grep "linux.dag:15\|linux.dag:16\|linux.dag:17\|linux.dag:18\|linux.dag:19" /tmp/base_mw.log || echo "none in baseline"
grep "linux.dag:15\|linux.dag:16\|linux.dag:17\|linux.dag:18\|linux.dag:19" /tmp/fix_mw.log || echo "none in fixed"

echo "=== git diff stat of applied stage0 changes ==="
git diff --stat

echo "DONE"
