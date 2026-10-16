#!/bin/bash
# Regen the four generated stage0 mirrors via the heal.yml path:
#   target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2
# (.github/workflows/heal.yml step "Converge the stage0 mirrors to the seed's own fixed point").
# NOT claim_batch --required-regen: that is the GATE, and its host-budget source refuses
# HostBudgetUnreadable on this runner (no cgroup v2 memory.max, no root).
# Prints the 4 candidate mirrors base64'd between markers so they can be staged locally.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
echo "HEAD=$(git rev-parse HEAD)"
echo "--- merge in progress on runner? (should be NO; mirrors come via patch) ---"
ls "$(git rev-parse --git_dir)" | grep -c MERGE_HEAD || true
for f in src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs; do
  echo "PRE sha=$(sha256sum "$f" | cut -c1-16) UnifyGenericsAcc=$(grep -c UnifyGenericsAcc "$f") $f"
done
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -3
BIN=target/release/claim_executor
[ -x "$BIN" ] || { echo "NO claim_executor"; exit 2; }
echo "===== regen: $BIN --regen-round-cost --source-root dag --source-root src/v2 ====="
"$BIN" --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen_round.log 2>&1
rc=$?
echo "regen rc=$rc"
tail -20 /tmp/regen_round.log
echo "===== budget error verbatim if present ====="
grep -E "HostBudget|host budget" /tmp/regen_round.log | head -3
echo "===== git status after regen ====="
git status --porcelain
echo "===== POST mirror hashes ====="
for f in src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs; do
  echo "POST sha=$(sha256sum "$f" | cut -c1-16) UnifyGenericsAcc=$(grep -c UnifyGenericsAcc "$f") $f"
done
echo "=== BEGIN-TAR-BASE64 ==="
tar -c src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs 2>/dev/null | base64 -w0
echo "=== END-TAR-BASE64 ==="
echo "RETRY_DONE rc=$rc"
