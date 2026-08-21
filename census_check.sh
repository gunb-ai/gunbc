set -euo pipefail
cargo build --release -p v1-compiler --bin gunbc >/dev/null 2>&1 || { echo BUILD_FAIL; exit 1; }
OUT=$(mktemp -d)
set +e
./target/release/gunbc compile --source-root dag --source-root src/v1 --source-root src/v2 \
  --entry src/v1/tests/claim/carrier_realization_census.dag --output-dir "$OUT" --target rust \
  --dependency-pool-index primary-precedence > "$OUT/log" 2>&1
echo "CHECK_EXIT=$?"
set -e
grep -m1 "compiled:" "$OUT/log" || true
grep -iE "error|refus|unresolved|not found|mismatch" "$OUT/log" | head -25 || true
echo "===CHECK_END==="
