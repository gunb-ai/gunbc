#!/bin/bash
# Regen the 4 stage0 mirrors for THIS merged tree (.dag authorities = fold-carrying
# merge of origin/main 01af778 + PR B). Returns the mirrors base64'd so the merge
# resolution is the regen's product, not a hand pick. Also captures the FULL list of
# tracked files regen moved (parent msg_06639145: report, do not silently commit).
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
echo "HEAD=$(git rev-parse HEAD)"
if [ ! -x target/release/claim_executor ]; then
  cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2
fi
[ -x target/release/claim_executor ] || { echo "NO claim_executor"; exit 2; }
echo "PRE mirror hashes:"
for f in src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs; do
  echo "PRE sha=$(sha256sum "$f" | cut -c1-16) U=$(grep -c UnifyGenericsAcc "$f") $f"
done
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regenM.log 2>&1
rc=$?
echo "MERGED regen rc=$rc"
grep -E "refused|first_generation_equal|hard diagnostic|type mismatch" /tmp/regenM.log | head -8
tail -8 /tmp/regenM.log
echo "===== FULL tracked-file moves by regen (report, do not commit blindly) ====="
git status --porcelain | grep -v '^??' 
echo "===== POST mirror hashes ====="
for f in src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs; do
  echo "POST sha=$(sha256sum "$f" | cut -c1-16) U=$(grep -c UnifyGenericsAcc "$f") $f"
done
mkdir -p mirrors_merged
cp src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs mirrors_merged/ 2>/dev/null
echo "=== BEGIN-TAR-BASE64 ==="
tar -c mirrors_merged 2>/dev/null | base64 -w0
echo "=== END-TAR-BASE64 ==="
echo "ALLDONE rc=$rc"
