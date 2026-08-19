set -e
MAIN_SHA=9fb884aa0062754a482b1b0e85b491e989826330
FIX_SHA=ed235bd2598

echo "=== MAIN BASELINE PHASE (clean, no surface-ownership commits) ==="
git fetch origin $MAIN_SHA
git checkout --force $MAIN_SHA
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$MAIN_SHA" ]; then echo "MAINFAIL got $GOT want $MAIN_SHA"; exit 99; fi
echo "MAIN_OK $GOT"
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -5

echo "--- clean-main required-regen (should be a stable baseline; committed stage0 should already match main's own .dag) ---"
set +e
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 --regen-candidate-dir /tmp/main-regen-candidate --regen-receipt /tmp/main-regen-receipt.json > /tmp/main_rr.log 2>&1
echo "main_rr_exit=$?"
set -e
tail -20 /tmp/main_rr.log

echo "=== ISOLATED FIX BRANCH PHASE (cherry-pick onto clean main only) ==="
git fetch origin $FIX_SHA
git checkout --force $FIX_SHA
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$FIX_SHA" ]; then echo "FIXFAIL got $GOT want $FIX_SHA (fetch may need full ref)"; fi
echo "FIX_TREE_OK $(git rev-parse HEAD)"

echo "--- regenerate candidate stage0 rust from the isolated fix branch, using the clean-main-built binary ---"
set +e
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 --regen-candidate-dir /tmp/isolated-fix-regen-candidate --regen-receipt /tmp/isolated-fix-regen-receipt.json > /tmp/isolated_fix_rr.log 2>&1
echo "isolated_fix_rr_exit=$?"
set -e
tail -30 /tmp/isolated_fix_rr.log

echo "--- checking specifically for unresolved-symbol / missing-name errors (would indicate the cherry-pick silently depended on unrelated old-branch content) ---"
grep -iE "unresolved|unbound|unknown (symbol|name|function|type)|no such (field|method|function)|not found in scope|cannot find" /tmp/isolated_fix_rr.log || echo "NO unresolved-symbol errors found"

echo "--- diff candidate infer/lookup rust vs committed on this branch (confirm real, non-trivial emitted change, same as before) ---"
find /tmp/isolated-fix-regen-candidate -iname "*infer*.rs" -o -iname "*lookup*.rs" | while read f; do
  bn=$(basename "$f")
  committed=$(find src/v1/stage0/src -name "$bn")
  if [ -n "$committed" ]; then
    echo "=== diff: $bn ==="
    diff -q "$f" "$committed" || true
  fi
done

echo "--- apply ONLY infer/lookup candidate files, rebuild, confirm the isolated branch compiles standalone with the fix baked into stage0 ---"
for f in $(find /tmp/isolated-fix-regen-candidate -iname "*infer*.rs" -o -iname "*lookup*.rs"); do
  bn=$(basename "$f")
  target=$(find src/v1/stage0/src -name "$bn")
  if [ -n "$target" ]; then
    cp "$f" "$target"
    echo "applied $bn -> $target"
  fi
done
cargo fmt --all 2>&1 | tail -5 || true

echo "--- rebuild gunbc+claim_executor on the isolated fix branch with regenerated stage0 rust applied ---"
set +e
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -40
BUILD_EXIT=$?
set -e
echo "isolated_build_exit=$BUILD_EXIT"

echo "DONE"
